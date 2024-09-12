use std::fs::File;
use std::io::{self, BufRead};

// Define a struct to represent each record in the CSV
#[derive(Debug, PartialEq)]
struct Record {
    timestamp: String,
    sensor_id: String,
    sensor_reading: f64,
    unit: String,
    location: String,
    experiment_type: String,
}

// Function to simulate reading and parsing the CSV file line by line
fn read_csv(file_path: &str) -> Vec<Record> {
    let mut records = Vec::new();

    // Open the file
    if let Ok(file) = File::open(file_path) {
        // Read each line using a buffered reader
        let lines = io::BufReader::new(file).lines();

        for (index, line) in lines.enumerate() {
            if let Ok(record) = line {
                // Skip the header
                if index == 0 {
                    continue;
                }
                // Split the record by commas
                let fields: Vec<&str> = record.split(',').collect();

                if fields.len() == 6 {
                    let timestamp = fields[0].trim().to_string();
                    let sensor_id = fields[1].trim().to_string();
                    let sensor_reading = fields[2].trim().parse::<f64>().unwrap_or(0.0);
                    let unit = fields[3].trim().to_string();
                    let location = fields[4].trim().to_string();
                    let experiment_type = fields[5].trim().to_string();

                    // Push the parsed data as a Record object
                    records.push(Record {
                        timestamp,
                        sensor_id,
                        sensor_reading,
                        unit,
                        location,
                        experiment_type,
                    });
                }
            }
        }
    }
    records
}

// Function to simulate a data pipeline
fn data_pipeline(records: Vec<Record>) -> Vec<Record> {
    records
        .into_iter()
        // Example: filter out readings from 'Lab 2'
        .filter(|r| r.location != "Lab 2")
        // Example: convert all Celsius temperatures to Fahrenheit (for demonstration)
        .map(|mut r| {
            if r.unit == "Celsius" {
                r.sensor_reading = r.sensor_reading * 1.8 + 32.0;
                r.unit = "Fahrenheit".to_string();
            }
            r
        })
        .collect()
}

fn main() {
    let file_path = "scientific_data.csv";

    // Step 1: Read CSV file
    let records = read_csv(file_path);

    // Step 2: Process data pipeline (filter and transform)
    let processed_records = data_pipeline(records);

    // Step 3: Print out processed data
    for record in processed_records {
        println!(
            "Timestamp: {}, SensorID: {}, SensorReading: {}, Unit: {}, Location: {}, Experiment: {}",
            record.timestamp, record.sensor_id, record.sensor_reading, record.unit, record.location, record.experiment_type
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let csv_data = "\
        Timestamp, SensorID, SensorReading, Unit, Location, ExperimentType\n\
        2024-09-10 12:00, A12, 23.5, Celsius, Lab 1, Temperature Monitoring\n\
        2024-09-10 12:05, B22, 760, mmHg, Lab 2, Pressure Measurement\n";

        // Simulate reading from a CSV file by writing the data into a temporary file
        let test_file_path = "test_data.csv";
        std::fs::write(test_file_path, csv_data).unwrap();

        let records = read_csv(test_file_path);
        let expected = vec![
            Record {
                timestamp: "2024-09-10 12:00".to_string(),
                sensor_id: "A12".to_string(),
                sensor_reading: 23.5,
                unit: "Celsius".to_string(),
                location: "Lab 1".to_string(),
                experiment_type: "Temperature Monitoring".to_string(),
            },
            Record {
                timestamp: "2024-09-10 12:05".to_string(),
                sensor_id: "B22".to_string(),
                sensor_reading: 760.0,
                unit: "mmHg".to_string(),
                location: "Lab 2".to_string(),
                experiment_type: "Pressure Measurement".to_string(),
            },
        ];

        assert_eq!(records, expected);

        // Clean up the test file
        std::fs::remove_file(test_file_path).unwrap();
    }

    #[test]
    fn test_data_pipeline() {
        let input_data = vec![
            Record {
                timestamp: "2024-09-10 12:00".to_string(),
                sensor_id: "A12".to_string(),
                sensor_reading: 23.5,
                unit: "Celsius".to_string(),
                location: "Lab 1".to_string(),
                experiment_type: "Temperature Monitoring".to_string(),
            },
            Record {
                timestamp: "2024-09-10 12:05".to_string(),
                sensor_id: "B22".to_string(),
                sensor_reading: 760.0,
                unit: "mmHg".to_string(),
                location: "Lab 2".to_string(),
                experiment_type: "Pressure Measurement".to_string(),
            },
        ];

        let processed_records = data_pipeline(input_data);

        let expected = vec![Record {
            timestamp: "2024-09-10 12:00".to_string(),
            sensor_id: "A12".to_string(),
            sensor_reading: 74.3, // Converted from 23.5 Celsius to Fahrenheit
            unit: "Fahrenheit".to_string(),
            location: "Lab 1".to_string(),
            experiment_type: "Temperature Monitoring".to_string(),
        }];

        // Helper function to compare floats with a tolerance
        fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
            assert!(
                (a - b).abs() < epsilon,
                "Values not approximately equal: {} != {}",
                a,
                b
            );
        }

        for (actual, exp) in processed_records.iter().zip(expected.iter()) {
            assert_eq!(actual.timestamp, exp.timestamp);
            assert_eq!(actual.sensor_id, exp.sensor_id);
            assert_approx_eq(actual.sensor_reading, exp.sensor_reading, 1e-10); // Tolerance for floating-point comparison
            assert_eq!(actual.unit, exp.unit);
            assert_eq!(actual.location, exp.location);
            assert_eq!(actual.experiment_type, exp.experiment_type);
        }
    }
}
