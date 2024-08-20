use std::collections::{HashMap, VecDeque};

use bevy::math::VectorSpace;
use game::GameLayerOrder;

use crate::*;

#[derive(PartialEq, Eq)]
pub struct Path {
    pub points: Vec<UVec2>
}

impl Path {
    pub fn contains_cell(&self, cell_pos: &UVec2) -> bool {
        self.points.contains(cell_pos)
    }

    pub fn start(&self) -> &UVec2 {
        self.points.first().unwrap()
    }

    pub fn end(&self) -> &UVec2 {
        self.points.last().unwrap()
    }
}

pub enum PathState {
    NoChange,
    Updated
}

#[derive(Resource)]
pub struct GameGrid {
    pub data: Vec<Cell>,
    pub path: Option<Path>,
    pub portal_pos: Vec3,
    pub core_pos: Vec3
}

impl GameGrid {
    fn coords_to_index(&self, cell: &UVec2) -> Option<usize> {
        if cell.x < GRID_WIDTH && cell.y < GRID_HEIGHT {
            Some((cell.y * GRID_WIDTH + cell.x) as usize)
        } else {
            None
        }
    }

    pub fn get_cell(&self, cell: &UVec2) -> Option<&Cell> {
        self.coords_to_index(cell).map(|index| &self.data[index])
    }

    pub fn get_cell_mut(&mut self, cell: &UVec2) -> Option<&mut Cell> {
        self.coords_to_index(cell).map(|index| &mut self.data[index])
    }

    pub fn set_special(&mut self, cell_pos: &UVec2, special: SpecialCell) -> Option<&Cell> {
        let index = self.coords_to_index(cell_pos)?;
        let cell = &mut self.data[index];
            
        if cell.special.is_some() {
            error!("{} overlaps with another special cell at {}!", special, cell.position);
            return None;
        }

        cell.special = Some(special);
        match special {
            SpecialCell::Core => self.core_pos = cell_to_world_pos(cell_pos),
            SpecialCell::Portal =>{
                self.portal_pos = cell_to_world_pos(cell_pos);
                for neighbour_pos in &self.get_neighbors(cell_pos, true) {
                    self.get_cell_mut(neighbour_pos).unwrap().special = Some(SpecialCell::PortalSurrounding);
                }
            }
            _ => {}
        }

        Some(&self.data[index])
    }

    pub fn calculate_path(&mut self) -> PathState {
        let new_path = self.calculate_path_with_module_at(UVec2::MAX);

        if let Some(new_path) = new_path {
            if self.path.as_ref() != Some(&new_path) {
                self.path = Some(new_path);
                PathState::Updated
            } else {
                PathState::NoChange
            }
        } else {
            warn!("Failed to calculate path!");
            PathState::NoChange
        }
    }

    pub fn try_place_module(
        &mut self,
        cell_pos: &UVec2,
        commands: &mut Commands,
        game_textures: &GameTextures
    ) -> PathState {
        let Some(cell) = self.get_cell_mut(cell_pos) else { return PathState::NoChange };
    
        const MODULE: &Buildable = &Buildable::Standalone(StandaloneBuildable::Module);

        fn spawn_module(cell_pos: &UVec2, commands: &mut Commands, game_textures: &GameTextures) -> Entity {
            commands.spawn((
                SpriteBundle {
                    texture: game_textures.module.clone(),
                    transform: Transform {
                        translation: cell_to_world_pos(cell_pos).on(GameLayer::STANDALONE),
                        scale: Vec3::splat(0.5),
                        ..default()
                    },
                    ..default()
                },
                RenderLayers::layer(1)
            )).id()
        }
    
        // Cell is already occupied
        if !cell.can_place(MODULE) {

            // Cell is occupied by other module - remove it
            if let Some((entity, StandaloneBuildable::Module)) = cell.standalone_entity {
                cell.remove_standalone();
                commands.entity(entity).despawn();
    
                info!("Removed Module at {}", cell_pos);

                // Module could be blocking better path - not skipping the calulcation
                return self.calculate_path();
            }
            return PathState::NoChange
        }

        // New module is not on the track - no update needed
        if self.path.as_ref().is_some_and(|path| !path.contains_cell(cell_pos)) {
            let module = spawn_module(cell_pos, commands, game_textures);
            self.get_cell_mut(cell_pos).unwrap().set_standalone(StandaloneBuildable::Module, module);

            debug!("Skipped path calculation");
            info!("Placed Module at {}", cell_pos);
            return PathState::NoChange
        }
    
        // Verifying that module location is legal 
        let path_result = self.get_cell(cell_pos)
            .and_then(|cell| self.calculate_path_with_module_at(cell.position));
    
        if let Some(path) = path_result {
            self.path = Some(path);
    
            let module = spawn_module(cell_pos, commands, game_textures);
            self.get_cell_mut(cell_pos).unwrap().set_standalone(StandaloneBuildable::Module, module);
    
            info!("Placed Module at {}", cell_pos);
            return PathState::Updated
        } else {
            info!("Can't block path to the Core");
        }

        PathState::NoChange
    }

    pub fn can_place(&self, cell: &UVec2, buildable: &Buildable) -> bool {
        let can_place = self.coords_to_index(cell).map(|index| self.data[index].can_place(buildable))
            .unwrap_or(false);

        if buildable.is_module() && can_place {
            self.calculate_path_with_module_at(*cell).is_some()
        } else {
            can_place
        }
    }

    pub fn calculate_path_with_module_at(&self, module_pos: UVec2) -> Option<Path> {
        let portal_pos = self.find_special_cell(SpecialCell::Portal)?;
        let core_pos = self.find_special_cell(SpecialCell::Core)?;

        let mut queue = VecDeque::new();
        let mut visited = vec![false; (GRID_WIDTH * GRID_HEIGHT) as usize];
        let mut parent = HashMap::new();

        queue.push_back(portal_pos);
        visited[self.coords_to_index(&portal_pos)?] = true;

        while let Some(current) = queue.pop_front() {
            if current == core_pos {
                let mut path = vec![core_pos];
                let mut current_pos = current;

                while let Some(&prev) = parent.get(&current_pos) {
                    path.push(prev);
                    current_pos = prev;
                }

                path.reverse();
                return Some(Path { points: path });
            }

            for neighbor in self.get_neighbors(&current, false) {
                let index = self.coords_to_index(&neighbor)?;

                if !visited[index] && neighbor != module_pos && self.can_move_to(&neighbor) {
                    visited[index] = true;
                    queue.push_back(neighbor);
                    parent.insert(neighbor, current);
                }
            }
        }

        None
    }

    pub fn get_neighbors(&self, cell: &UVec2, include_diagonals: bool) -> Vec<UVec2> {
        let mut neighbors = Vec::new();

        if include_diagonals {
            for dx in [-1, 0, 1].iter() {
                for dy in [-1, 0, 1].iter() {
                    if *dx == 0 && *dy == 0 {
                        continue;
                    }

                    let new_x = cell.x as i32 + dx;
                    let new_y = cell.y as i32 + dy;

                    if new_x >= 0 && new_x < GRID_WIDTH as i32 && new_y >= 0 && new_y < GRID_HEIGHT as i32 {
                        neighbors.push(UVec2::new(new_x as u32, new_y as u32));
                    }
                }
            }
        } else {
            if cell.x > 0 {
                neighbors.push(UVec2::new(cell.x - 1, cell.y));
            }
            if cell.x < GRID_WIDTH - 1 {
                neighbors.push(UVec2::new(cell.x + 1, cell.y));
            }
            if cell.y > 0 {
                neighbors.push(UVec2::new(cell.x, cell.y - 1));
            }
            if cell.y < GRID_HEIGHT - 1 {
                neighbors.push(UVec2::new(cell.x, cell.y + 1));
            }
        }

        neighbors
    }

    fn can_move_to(&self, cell: &UVec2) -> bool {
        self.get_cell(cell).map_or(false, |c| !c.has_moudle())
    }

    fn find_special_cell(&self, special: SpecialCell) -> Option<UVec2> {
        let special_cell = self.data.iter()
            .find(|cell| cell.special == Some(special))
            .map(|cell| cell.position);

        if special_cell.is_none() {
            error!("Unable to find {} in the grid!", special);
        }

        special_cell
    }
}

impl Default for GameGrid {
    fn default() -> Self {
        let mut data = Vec::with_capacity((GRID_WIDTH * GRID_HEIGHT) as usize);

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let position = UVec2::new(x, y);
                data.push(Cell {
                    position,
                    ..default()
                });
            }
        }

        Self {
            data,
            path: None,
            core_pos: Vec3::ZERO,
            portal_pos: Vec3::ZERO
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Cell {
    pub position: UVec2,
    pub standalone_entity: Option<(Entity, StandaloneBuildable)>,
    pub turret_entity: Option<(Entity, Turret)>,
    pub special: Option<SpecialCell>
}

#[derive(Clone, Copy, PartialEq, Eq, Display)]
pub enum SpecialCell {
    Core,
    Portal,
    PortalSurrounding
}

impl Cell {

    pub fn world_pos(&self) -> Vec3 {
        cell_to_world_pos(&self.position)
    }

    pub fn set_standalone(&mut self, standalone: StandaloneBuildable, entity: Entity) {
        self.standalone_entity = Some((entity, standalone));
    }

    pub fn set_turret(&mut self, turret: Turret, entity: Entity) {
        if self.standalone_entity.is_none() {
            warn!("Turret without module at {}", self.position);
        }

        self.turret_entity = Some((entity, turret));
    }

    pub fn is_empty(&self) -> bool {
        self.standalone_entity.is_none() && self.special.is_none()
    }

    pub fn has_moudle(&self) -> bool {
        self.standalone_entity.is_some_and(|(_, a)| matches!(a, StandaloneBuildable::Module))
    }

    pub fn is_special(&self) -> bool {
        self.special.is_some()
    }

    pub fn remove_standalone(&mut self) {
        if self.turret_entity.is_some() {
            warn!("Turret without module at {}", self.position);
        }

        self.standalone_entity = None;
    }

    pub fn remove_turret(&mut self) {
        self.turret_entity = None;
    }

    pub fn can_place(&self, buildable: &Buildable) -> bool {
        match buildable {
            Buildable::Standalone(_) => self.is_empty(),
            Buildable::Turret(_) => {
                if let Some((_, StandaloneBuildable::Module)) = self.standalone_entity {
                    self.turret_entity.is_none()
                } else {
                    false
                }
            }
        }
    }
}
