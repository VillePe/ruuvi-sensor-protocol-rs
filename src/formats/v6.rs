use crate::formats::{
    traits::{
        Acceleration, BatteryPotential, Humidity, MacAddress, MeasurementSequenceNumber,
        MovementCounter, Pressure, ProtocolPayload, Temperature, TransmitterPower,
        Pm25, Co2, Voc, Nox, DataFormat, Lux, AirDensity
    },
    AccelerationVector,
};
use crate::utils;

/// Raw sensor values parsed from manufacturer data.
#[derive(Debug, Eq, PartialEq)]
pub struct SensorValues {
    humidity: u16,
    temperature: i16,
    pressure: u16,
    measurement_sequence_number: u8,
    mac_address_3_lowest: [u8; 3],
    pm25: u16,
    co2: u16,
    voc : u16,
    nox : u16,
    lux : u8,
    calibration_in_progress: bool,
}

impl Humidity for SensorValues {
    fn humidity_as_ppm(&self) -> Option<u32> {
        if self.humidity == 0xFFFF {
            None
        } else {
            Some(u32::from(self.humidity) * 25)
        }
    }
}

impl MeasurementSequenceNumber for SensorValues {
    fn  measurement_sequence_number(&self) -> Option<u32> {
        // Some(1)
        Some(u32::from(self.measurement_sequence_number))
    }
}

impl Pressure for SensorValues {
    fn pressure_as_pascals(&self) -> Option<u32> {
        if self.pressure == 0xFFFF {
            None
        } else {
            Some(u32::from(self.pressure) + 50_000)
        }
    }
}

impl Temperature for SensorValues {
    fn temperature_as_millikelvins(&self) -> Option<u32> {
        if self.temperature == i16::MIN {
            None
        } else {
            let temperature = i32::from(self.temperature) * 5;

            #[expect(
                clippy::as_conversions,
                clippy::cast_possible_wrap,
                clippy::cast_sign_loss
            )]
            let temperature = (Self::ZERO_CELSIUS_IN_MILLIKELVINS as i32 + temperature) as u32;

            Some(temperature)
        }
    }
}

impl Pm25 for SensorValues {
        fn pm25_as_10micrograms_per_cubicmeter(&self) -> Option<u16> {
            if self.pm25 == 0xFFFF {
                None
            } else {
                Some(self.pm25)
            }
    }
}

impl Co2 for SensorValues {
    fn co2_as_ppm(&self) -> Option<u16> {
        if self.co2 == 0xFFFF {
            None
        } else {
            Some(self.co2)
        }
    }
}

impl Voc for SensorValues {
    fn voc_index(&self) -> Option<u16> {
        if self.voc & 0x01 == 1 {
            None
        } else {
            Some(self.voc)
        }
    }
}

impl Nox for SensorValues {
    fn nox_index(&self) -> Option<u16> {
        if self.nox & 0x01 == 1 {
            None
        } else {
            Some(self.nox)
        }
    }
}

impl Lux for SensorValues {
    fn lux_as_logarithmic_value(&self) -> Option<u8> {
        if self.lux == 0xFF {
            None
        } else {
            Some(self.lux)
        }
    }
}

impl Acceleration for SensorValues {
    fn acceleration_vector_as_milli_g(&self) -> Option<AccelerationVector> {
        None
    }
}

impl BatteryPotential for SensorValues {
    fn battery_potential_as_millivolts(&self) -> Option<u16> {
        None
    }
}

impl MacAddress for SensorValues {
    fn mac_address(&self) -> Option<[u8; 6]> {
        None
    }
}

impl MovementCounter for SensorValues {
    fn movement_counter(&self) -> Option<u32> {
        None
    }
}

impl TransmitterPower for SensorValues {
    fn tx_power_as_dbm(&self) -> Option<i8> {
        None
    }
}

impl DataFormat for SensorValues {
    fn get_dataformat(&self) -> Option<u8> {
        Some(6)
    }
}

impl AirDensity for SensorValues {
    fn get_air_density_grams_per_cubic_meter(&self) -> Option<u16> {
        Some((utils::calculate_air_density(
            self.temperature_as_millicelsius()? as f32/1000.0,
            self.humidity_as_ppm()? as f32 / 10_000.0,
            self.pressure_as_pascals()? as f32) * 1000.0) as u16)
    }
}
impl ProtocolPayload for SensorValues {
    const VERSION: u8 = 6;
    const SIZE: usize = 19;
}

impl From<&[u8; Self::SIZE]> for SensorValues {
    #[expect(clippy::similar_names)]
    fn from(value: &[u8; Self::SIZE]) -> Self {
        let [temperature_1, temperature_2, humidity_1, humidity_2,
            pressure_1, pressure_2, pm_1, pm_2, co2_1, co2_2, voc, nox,
            lux, _reserved, measurement_sequence_number_1, flags,
            mac_1, mac_2, mac_3]
            = value;
        let mut result = Self {
            temperature: i16::from_be_bytes([*temperature_1, *temperature_2]),
            humidity: u16::from_be_bytes([*humidity_1, *humidity_2]),
            pressure: u16::from_be_bytes([*pressure_1, *pressure_2]),
            pm25: u16::from_be_bytes([*pm_1, *pm_2]),
            co2: u16::from_be_bytes([*co2_1, *co2_2]),
            voc: u16::from(*voc) << 1,
            nox: u16::from(*nox) << 1,
            lux: *lux,
            measurement_sequence_number: *measurement_sequence_number_1,
            mac_address_3_lowest: [*mac_1, *mac_2, *mac_3],
            calibration_in_progress: false,
        };
        if flags & 0x40 == 0x40 {
            result.voc = result.voc + 1;
        }
        if flags & 0x80 == 0x80 {
            result.nox = result.nox + 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::formats::testing::test_measurement_trait_methods;

    // These test vectors are from the protocol specification
    // https://docs.ruuvi.com/communication/bluetooth-advertisements/data-format-6
    const VALID_VALUES: [u8; SensorValues::SIZE] = [
        // Temp     Humid       Pressure    PM25        CO2         VOC   NOX   LUX   Res   MSeq
        0x17, 0x0C, 0x56, 0x68, 0xC7, 0x9E, 0x00, 0x70, 0x00, 0xC9, 0x05, 0x01, 0xD9, 0x00, 0xCD,
        //Fl  Low 3 bytes mac
        0x00, 0x4C, 0x88, 0x4F
    ];

    const MAX_VALUES: [u8; SensorValues::SIZE] = [
        // Temp     Humid       Pressure    PM25        CO2         VOC   NOX   LUX   Res   MSeq
        0x7F, 0xFF, 0x9C, 0x40, 0xFF, 0xFE, 0x27, 0x10, 0x9C, 0x40, 0xFA, 0xFA, 0xFE, 0x00, 0xFF,
        //Fl  Low 3 bytes mac
        0x07, 0x4C, 0x8F, 0x4F
    ];

    const MIN_VALUES: [u8; SensorValues::SIZE] = [
        0x80, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00
    ];

    const INVALID_VALUES: [u8; SensorValues::SIZE] = [
        // Temp     Humid       Pressure    PM25        CO2         VOC   NOX   LUX   Res   MSeq
        0x80, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF,
        //Fl  Low 3 bytes mac
        0xFF, 0xFF, 0xFF, 0xFF
    ];

    const INVALID_VOC_VALUES: [u8; SensorValues::SIZE] = [
        // Temp     Humid       Pressure    PM25        CO2         VOC   NOX   LUX   Res   MSeq
        0x80, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x05, 0x01, 0xFF, 0x00, 0xFF,
        //Fl  Low 3 bytes mac
        0b0100_0000, 0xFF, 0xFF, 0xFF
    ];

    const INVALID_NOX_VALUES: [u8; SensorValues::SIZE] = [
        // Temp     Humid       Pressure    PM25        CO2         VOC   NOX   LUX   Res   MSeq
        0x80, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x05, 0x01, 0xFF, 0x00, 0xFF,
        //Fl  Low 3 bytes mac
        0b1000_0000, 0xFF, 0xFF, 0xFF
    ];

    #[test]
    fn valid_input() {
        assert_eq!(
            SensorValues::from(&VALID_VALUES),
            SensorValues {
                humidity: 0x5668,
                temperature: 0x170C,
                pressure: 0xC79E,
                measurement_sequence_number: 0xCD,
                pm25: 0x0070,
                co2: 0x00C9,
                voc: 0x05 << 1,
                nox: 0x01 << 1,
                lux: 0xD9,
                mac_address_3_lowest: [0x4C, 0x88, 0x4F],
                calibration_in_progress: false,
            }
        );
    }

    test_measurement_trait_methods! {
        test valid_values {
            values: SensorValues::from(&VALID_VALUES),
            expected: {
                acceleration_vector_as_milli_g: None,
                battery_potential_as_millivolts: None,
                humidity_as_ppm: Some(553_000),
                mac_address: None,
                measurement_sequence_number: Some(205),
                movement_counter: None,
                pressure_as_pascals: Some(101_102),
                temperature_as_millicelsius: Some(29_500),
                tx_power_as_dbm: None,
                pm25_as_micrograms_per_cubicmeter: Some(11.2),
                co2_as_ppm: Some(201),
                voc_index: Some(10),
                nox_index: Some(2),
                lux_as_logarithmic_value: Some(217),
                lux_as_normalized_value: Some(13_026.68),
            },
        }

        test invalid_values {
            values: SensorValues::from(&INVALID_VALUES),
            expected: {
                acceleration_vector_as_milli_g: None,
                battery_potential_as_millivolts: None,
                humidity_as_ppm: None,
                mac_address: None,
                measurement_sequence_number: Some(0xFF),
                movement_counter: None,
                pressure_as_pascals:None,
                temperature_as_millicelsius: None,
                tx_power_as_dbm: None,
                pm25_as_micrograms_per_cubicmeter: None,
                co2_as_ppm: None,
                voc_index: None,
                nox_index: None,
                lux_as_logarithmic_value: None,
                lux_as_normalized_value: None,
            },
        }

        test invalid_voc_values {
            values: SensorValues::from(&INVALID_VOC_VALUES),
            expected: {
                voc_index: None,
                nox_index: Some(2),
            },
        }

        test invalid_nox_values {
            values: SensorValues::from(&INVALID_NOX_VALUES),
            expected: {
                voc_index: Some(10),
                nox_index: None,
            },
        }

        test min_values {
            values: SensorValues::from(&MIN_VALUES),
            expected: {
                acceleration_vector_as_milli_g: None,
                battery_potential_as_millivolts: None,
                humidity_as_ppm: Some(0),
                mac_address: None,
                measurement_sequence_number: Some(0),
                movement_counter: None,
                pressure_as_pascals: Some(50_000),
                temperature_as_millicelsius: Some(-163_835),
                tx_power_as_dbm: None,
                pm25_as_micrograms_per_cubicmeter: Some(0.0),
                co2_as_ppm: Some(0),
                voc_index: Some(0),
                nox_index: Some(0),
                lux_as_logarithmic_value: Some(0),
                lux_as_normalized_value: Some(0.0),
            },
        }

        test max_values {
            values: SensorValues::from(&MAX_VALUES),
            expected: {
                acceleration_vector_as_milli_g: None,
                battery_potential_as_millivolts: None,
                humidity_as_ppm: Some(1_000_000),
                mac_address: None,
                measurement_sequence_number: Some(0xFF),
                movement_counter: None,
                pressure_as_pascals: Some(115_534),
                temperature_as_millicelsius: Some(163_835),
                tx_power_as_dbm: None,
                pm25_as_micrograms_per_cubicmeter: Some(1000.0),
                co2_as_ppm: Some(40000),
                voc_index: Some(500),
                nox_index: Some(500),
                lux_as_logarithmic_value: Some(254),
                lux_as_normalized_value: Some(65535.0),
            },
        }
    }
}
