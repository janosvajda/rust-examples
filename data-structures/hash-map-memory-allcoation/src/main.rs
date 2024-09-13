use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, PartialEq)]
struct UserProfile {
    name: String,
    age: u8,
    email: String,
    hobbies: Vec<String>,
    attributes: HashMap<String, String>,
}

fn create_user_profile(i: usize) -> UserProfile {
    let mut attributes = HashMap::new();
    attributes.insert("Membership".to_string(), "Gold".to_string());
    attributes.insert("Location".to_string(), "Unknown".to_string());

    UserProfile {
        name: format!("User{}", i),
        age: (i % 100) as u8, // Random age between 0 and 99
        email: format!("user{}@example.com", i),
        hobbies: vec![format!("Hobby{}", i % 10)], // Repeating hobbies
        attributes,
    }
}

fn main() {
    // Number of objects to create
    let num_objects = 1000000; // Adjust based on your system's capability

    // Measure time for allocation
    let start = Instant::now();
    let mut profiles: Vec<UserProfile> = Vec::with_capacity(num_objects);
    for i in 0..num_objects {
        let profile = create_user_profile(i);
        profiles.push(profile);
    }
    let allocation_duration = start.elapsed();

    // Measure time for deallocation
    let start = Instant::now();
    drop(profiles); // Explicitly drop the profiles to measure deallocation time
    let deallocation_duration = start.elapsed();

    println!("Time to allocate {} user profiles: {:?}", num_objects, allocation_duration);
    println!("Time to deallocate profiles: {:?}", deallocation_duration);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_creation() {
        let profile = create_user_profile(0);

        let mut expected_attributes = HashMap::new();
        expected_attributes.insert("Membership".to_string(), "Gold".to_string());
        expected_attributes.insert("Location".to_string(), "Unknown".to_string());

        assert_eq!(
            profile,
            UserProfile {
                name: "User0".to_string(),
                age: 0,
                email: "user0@example.com".to_string(),
                hobbies: vec!["Hobby0".to_string()],
                attributes: expected_attributes,
            }
        );
    }

    #[test]
    fn test_allocation_time() {
        let num_objects = 10_000; // A smaller number for testing purposes

        // Measure time for allocation
        let start = Instant::now();
        let mut profiles: Vec<UserProfile> = Vec::with_capacity(num_objects);
        for i in 0..num_objects {
            let profile = create_user_profile(i);
            profiles.push(profile);
        }
        let allocation_duration = start.elapsed();
        
        // Check that allocation took a reasonable time
        assert!(allocation_duration.as_secs_f64() < 1.0, "Allocation took too long!");

        // Deallocate and check the time
        let start = Instant::now();
        drop(profiles); // Explicitly drop the profiles to measure deallocation time
        let deallocation_duration = start.elapsed();

        // Ensure deallocation is fast as well
        assert!(deallocation_duration.as_secs_f64() < 1.0, "Deallocation took too long!");
    }
}
