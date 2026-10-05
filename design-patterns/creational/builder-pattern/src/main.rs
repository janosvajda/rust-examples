// Builder pattern: gather named options, then move them into a valid spacecraft.
use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
enum Assembler {
    Human,
    Robot,
    Nanorobots,
}

#[derive(Debug, PartialEq)]
struct Spacecraft {
    spacecraft_type: String,
    equipment: String,
    assembler: Assembler,
}

#[derive(Debug, PartialEq)]
struct BuildError(&'static str);
impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for BuildError {}

struct SpacecraftBuilder {
    spacecraft_type: String,
    equipment: Option<String>,
    assembler: Assembler,
}
impl SpacecraftBuilder {
    fn new(spacecraft_type: impl Into<String>) -> Self {
        Self {
            spacecraft_type: spacecraft_type.into(),
            equipment: None,
            assembler: Assembler::Human,
        }
    }
    fn equipment(mut self, equipment: impl Into<String>) -> Self {
        self.equipment = Some(equipment.into());
        self
    }
    fn assembler(mut self, assembler: Assembler) -> Self {
        self.assembler = assembler;
        self
    }
    fn build(self) -> Result<Spacecraft, BuildError> {
        if self.spacecraft_type.trim().is_empty() {
            return Err(BuildError("spacecraft type is required"));
        }
        let equipment = self
            .equipment
            .filter(|s| !s.trim().is_empty())
            .ok_or(BuildError("equipment is required"))?;
        Ok(Spacecraft {
            spacecraft_type: self.spacecraft_type,
            equipment,
            assembler: self.assembler,
        })
    }
    fn shuttle() -> Self {
        Self::new("Space Shuttle").equipment("Basic space equipment")
    }
    fn battleship() -> Self {
        Self::new("Battleship")
            .equipment("Advanced space equipment")
            .assembler(Assembler::Robot)
    }
    fn dreadnought() -> Self {
        Self::new("Dreadnought")
            .equipment("Superior space equipment")
            .assembler(Assembler::Nanorobots)
    }
}

fn main() -> Result<(), BuildError> {
    let fleet = [
        SpacecraftBuilder::shuttle().build()?,
        SpacecraftBuilder::battleship().build()?,
        SpacecraftBuilder::dreadnought().build()?,
        SpacecraftBuilder::new("Research Shuttle")
            .equipment("Deep space telescope")
            .assembler(Assembler::Nanorobots)
            .build()?,
    ];
    for ship in fleet {
        println!(
            "{}: {}, assembled by {:?}",
            ship.spacecraft_type, ship.equipment, ship.assembler
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_blank_required_fields_are_errors() {
        assert_eq!(
            SpacecraftBuilder::new("Shuttle").build(),
            Err(BuildError("equipment is required"))
        );
        assert!(SpacecraftBuilder::new(" ")
            .equipment("Radar")
            .build()
            .is_err());
        assert!(SpacecraftBuilder::new("Shuttle")
            .equipment(" ")
            .build()
            .is_err());
    }
    #[test]
    fn presets_can_be_overridden_before_building() {
        let ship = SpacecraftBuilder::battleship()
            .equipment("Telescope")
            .assembler(Assembler::Human)
            .build()
            .unwrap();
        assert_eq!(ship.spacecraft_type, "Battleship");
        assert_eq!(ship.equipment, "Telescope");
        assert_eq!(ship.assembler, Assembler::Human);
    }
}
