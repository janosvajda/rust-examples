# Builder Pattern

The Builder pattern is a creational design pattern that separates the construction of a complex object from its representation, allowing the same construction process to create different representations of the object. This pattern is particularly useful when dealing with complex objects that have multiple variations and configurations, as it provides a flexible and organized way to build such objects step-by-step.

In the Builder pattern, we have the following key components:

1. **Product**: Represents the complex object that we want to construct. It contains all the components and details that the Builder constructs.

2. **Builder**: An abstract interface that declares methods for setting different parts of the Product. Concrete Builders implement this interface to provide specific implementations for building different variations of the Product.

3. **Director**: Responsible for managing the construction process. It works with the Builder to construct the Product step-by-step based on a specific algorithm or process.

4. **Concrete Builder**: Implements the Builder interface to build a specific variation of the Product. It contains methods for setting the attributes of the Product.

The Builder pattern allows us to create complex objects by combining and configuring different components step-by-step, making it easier to manage complex object creation without exposing its internal representation.

# Example: Building Different Types of Spacecraft

In this example, we have demonstrated the Builder pattern to construct three different types of spacecraft: Space Shuttle, Battleship, and Dreadnought. Each type of spacecraft has its own set of equipment and an assembler responsible for constructing it.

1. **SpaceShuttleBuilder**: Implements the Builder trait to construct a Space Shuttle spacecraft. It sets the spacecraft type to "Space Shuttle," default equipment to "Basic space equipment," and default assembler to "Human."

2. **BattleshipBuilder**: Implements the Builder trait to construct a Battleship spacecraft. It sets the spacecraft type to "Battleship," default equipment to "Advanced space equipment," and default assembler to "Robot."

3. **DreadnoughtBuilder**: Implements the Builder trait to construct a Dreadnought spacecraft. It sets the spacecraft type to "Dreadnought," default equipment to "Superior space equipment," and default assembler to "Nanorobots."

The Director, represented by the `SpacecraftDirector` struct, is responsible for orchestrating the construction process. It takes a specific builder (e.g., `SpaceShuttleBuilder`, `BattleshipBuilder`, or `DreadnoughtBuilder`) as input and uses it to build the corresponding spacecraft step-by-step.

Each builder sets the properties of the spacecraft and constructs it using the provided data. Finally, the constructed spacecraft is printed using the `print_details()` method of the `Spacecraft` struct.

The example demonstrates how the Builder pattern allows us to construct different types of spacecraft with various attributes and configurations, providing a clear separation between the construction process and the resulting objects.
