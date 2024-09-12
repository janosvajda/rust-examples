// Define the Spacecraft struct to represent a spacecraft with its properties.
struct Spacecraft {
    // Define spacecraft properties here
    spacecraft_type: String, // The type of the spacecraft (e.g., Space Shuttle, Battleship, Dreadnought)
    equipment: String,       // The equipment used by the spacecraft
    assembler: String,       // The type of assembler used for spacecraft construction (e.g., Human, Robot, Nanorobots)
}

impl Spacecraft {
    // Method to print the details of the spacecraft.
    fn print_details(&self) {
        println!(
            "Spacecraft Type: {}\nEquipment: {}\nAssembler: {}\n",
            self.spacecraft_type, self.equipment, self.assembler
        );
    }
}

// Define the Builder trait to enforce a common interface for different spacecraft builders.
trait SpacecraftBuilder {
    fn set_spacecraft_type(&mut self, spacecraft_type: String);
    fn set_equipment(&mut self, equipment: String);
    fn set_assembler(&mut self, assembler: String);
    fn build(&self) -> Spacecraft;
}

// Implement the Space Shuttle builder to create a Space Shuttle spacecraft.
struct SpaceShuttleBuilder {
    spacecraft_type: String,
    equipment: String,
    assembler: String,
}

impl SpaceShuttleBuilder {
    // Create a new instance of the Space Shuttle builder with default values.
    fn new() -> Self {
        Self {
            spacecraft_type: String::from("Space Shuttle"),
            equipment: String::from("Basic space equipment"),
            assembler: String::from("Human"),
        }
    }
}

impl SpacecraftBuilder for SpaceShuttleBuilder {
    // Set the type of the spacecraft being built.
    fn set_spacecraft_type(&mut self, spacecraft_type: String) {
        self.spacecraft_type = spacecraft_type;
    }

    // Set the equipment used by the spacecraft being built.
    fn set_equipment(&mut self, equipment: String) {
        self.equipment = equipment;
    }

    // Set the type of assembler used for spacecraft construction.
    fn set_assembler(&mut self, assembler: String) {
        self.assembler = assembler;
    }

    // Build the Space Shuttle spacecraft using the provided properties.
    fn build(&self) -> Spacecraft {
        Spacecraft {
            spacecraft_type: self.spacecraft_type.clone(),
            equipment: self.equipment.clone(),
            assembler: self.assembler.clone(),
        }
    }
}

// Implement the Battleship builder to create a Battleship spacecraft.
struct BattleshipBuilder {
    spacecraft_type: String,
    equipment: String,
    assembler: String,
}

impl BattleshipBuilder {
    // Create a new instance of the Battleship builder with default values.
    fn new() -> Self {
        Self {
            spacecraft_type: String::from("Battleship"),
            equipment: String::from("Advanced space equipment"),
            assembler: String::from("Robot"),
        }
    }
}

impl SpacecraftBuilder for BattleshipBuilder {
    // Set the type of the spacecraft being built.
    fn set_spacecraft_type(&mut self, spacecraft_type: String) {
        self.spacecraft_type = spacecraft_type;
    }

    // Set the equipment used by the spacecraft being built.
    fn set_equipment(&mut self, equipment: String) {
        self.equipment = equipment;
    }

    // Set the type of assembler used for spacecraft construction.
    fn set_assembler(&mut self, assembler: String) {
        self.assembler = assembler;
    }

    // Build the Battleship spacecraft using the provided properties.
    fn build(&self) -> Spacecraft {
        Spacecraft {
            spacecraft_type: self.spacecraft_type.clone(),
            equipment: self.equipment.clone(),
            assembler: self.assembler.clone(),
        }
    }
}

// Implement the Dreadnought builder to create a Dreadnought spacecraft.
struct DreadnoughtBuilder {
    spacecraft_type: String,
    equipment: String,
    assembler: String,
}

impl DreadnoughtBuilder {
    // Create a new instance of the Dreadnought builder with default values.
    fn new() -> Self {
        Self {
            spacecraft_type: String::from("Dreadnought"),
            equipment: String::from("Superior space equipment"),
            assembler: String::from("Nanorobots"),
        }
    }
}

impl SpacecraftBuilder for DreadnoughtBuilder {
    // Set the type of the spacecraft being built.
    fn set_spacecraft_type(&mut self, spacecraft_type: String) {
        self.spacecraft_type = spacecraft_type;
    }

    // Set the equipment used by the spacecraft being built.
    fn set_equipment(&mut self, equipment: String) {
        self.equipment = equipment;
    }

    // Set the type of assembler used for spacecraft construction.
    fn set_assembler(&mut self, assembler: String) {
        self.assembler = assembler;
    }

    // Build the Dreadnought spacecraft using the provided properties.
    fn build(&self) -> Spacecraft {
        Spacecraft {
            spacecraft_type: self.spacecraft_type.clone(),
            equipment: self.equipment.clone(),
            assembler: self.assembler.clone(),
        }
    }
}

// Builder pattern director
struct SpacecraftDirector;

impl SpacecraftDirector {
    // Construct a spacecraft using the provided builder.
    fn construct(builder: &mut dyn SpacecraftBuilder) -> Spacecraft {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_shuttle_builder() {
        // Create a Space Shuttle builder with specific properties.
        let mut space_shuttle_builder = SpaceShuttleBuilder::new();
        space_shuttle_builder.set_equipment("Advanced space equipment".to_string());
        space_shuttle_builder.set_assembler("Robot".to_string());

        // Construct the Space Shuttle spacecraft.
        let spacecraft = SpacecraftDirector::construct(&mut space_shuttle_builder);

        // Assert that the constructed spacecraft has the correct properties.
        assert_eq!(spacecraft.spacecraft_type, "Space Shuttle");
        assert_eq!(spacecraft.equipment, "Advanced space equipment");
        assert_eq!(spacecraft.assembler, "Robot");
    }

    #[test]
    fn test_battleship_builder() {
        // Create a Battleship builder with specific properties.
        let mut battleship_builder = BattleshipBuilder::new();
        battleship_builder.set_assembler("Nanorobots".to_string());

        // Construct the Battleship spacecraft.
        let spacecraft = SpacecraftDirector::construct(&mut battleship_builder);

        // Assert that the constructed spacecraft has the correct properties.
        assert_eq!(spacecraft.spacecraft_type, "Battleship");
        assert_eq!(spacecraft.equipment, "Advanced space equipment");
        assert_eq!(spacecraft.assembler, "Nanorobots");
    }

    #[test]
    fn test_dreadnought_builder() {
        // Create a Dreadnought builder with specific properties.
        let mut dreadnought_builder = DreadnoughtBuilder::new();
        dreadnought_builder.set_equipment("Superior space equipment".to_string());

        // Construct the Dreadnought spacecraft.
        let spacecraft = SpacecraftDirector::construct(&mut dreadnought_builder);

        // Assert that the constructed spacecraft has the correct properties.
        assert_eq!(spacecraft.spacecraft_type, "Dreadnought");
        assert_eq!(spacecraft.equipment, "Superior space equipment");
        assert_eq!(spacecraft.assembler, "Nanorobots"); // Note: Assembler is not changed for Dreadnought.
    }
}

fn main() {
    // Build Space Shuttle
    let mut space_shuttle_builder = SpaceShuttleBuilder::new();
    let space_shuttle = SpacecraftDirector::construct(&mut space_shuttle_builder);
    space_shuttle.print_details();

    // Build Battleship
    let mut battleship_builder = BattleshipBuilder::new();
    let battleship = SpacecraftDirector::construct(&mut battleship_builder);
    battleship.print_details();

    // Build Dreadnought
    let mut dreadnought_builder = DreadnoughtBuilder::new();
    let dreadnought = SpacecraftDirector::construct(&mut dreadnought_builder);
    dreadnought.print_details();
}