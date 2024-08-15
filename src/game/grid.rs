use crate::*;


#[derive(Resource)]
pub struct GameGrid {
    pub data: Vec<Cell>
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

    pub fn set_cell(&mut self, cell: &UVec2, data: Cell) {
        if let Some(index) = self.coords_to_index(cell) {
            self.data[index] = data;
        }
    }

    pub fn can_place(&self, cell: &UVec2, buildable: &Buildable) -> bool {
        self.coords_to_index(cell).map(|index| self.data[index].can_place(buildable))
            .unwrap_or(false)
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
                    ..Default::default()
                });
            }
        }

        Self { data }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Cell {
    pub position: UVec2,
    pub standalone_entity: Option<(Entity, StandaloneBuildable)>,
    pub turret_entity: Option<(Entity, Turret)>
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
        self.standalone_entity.is_none()
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
