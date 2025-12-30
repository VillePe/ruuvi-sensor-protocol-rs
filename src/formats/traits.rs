use crate::formats::AccelerationVector;
use crate::utils;

pub trait Acceleration {
    /// Returns a three-dimensional acceleration vector where each component is in milli-G if an
    /// acceleration measurement is available.
    fn acceleration_vector_as_milli_g(&self) -> Option<AccelerationVector>;
}

pub trait BatteryPotential {
    /// Returns battery potential as milli-volts
    fn battery_potential_as_millivolts(&self) -> Option<u16>;
}

pub trait Humidity {
    /// Returns relative humidity as parts per million
    fn humidity_as_ppm(&self) -> Option<u32>;
    fn absolute_humidity_as_grams_per_cubic_meter(
        &self,
        temperature_in_celsius: f32,
    ) -> Option<f32> {
        self.humidity_as_ppm().and_then(|humidity| {
            Some(utils::absolute_humidity_as_grams_per_cubic_meter(
                temperature_in_celsius,
                humidity as f32 / 10_000.0,
            ))
        })
    }
}

pub trait MacAddress {
    /// Returns the MAC address of the sensor if available.
    fn mac_address(&self) -> Option<[u8; 6]>;
}

pub trait MeasurementSequenceNumber {
    /// Returns the measurement sequence number if available. The maximum value is not specified.
    fn measurement_sequence_number(&self) -> Option<u32>;
}

pub trait MovementCounter {
    /// Returns the movement count of the tag if available. The maximum value is not specified.
    fn movement_counter(&self) -> Option<u32>;
}

pub trait Pressure {
    /// Returns pressure as pascals
    fn pressure_as_pascals(&self) -> Option<u32>;
}

pub trait Temperature {
    const ZERO_CELSIUS_IN_MILLIKELVINS: u32 = 273_150;

    /// Returns temperature as milli-kelvins if a temperature reading is available.
    fn temperature_as_millikelvins(&self) -> Option<u32>;

    /// Returns temperature as milli-Celsius if a temperature reading is available.
    fn temperature_as_millicelsius(&self) -> Option<i32> {
        let temperature = self.temperature_as_millikelvins()?;

        #[expect(clippy::as_conversions, clippy::cast_possible_wrap)]
        Some(temperature as i32 - Self::ZERO_CELSIUS_IN_MILLIKELVINS as i32)
    }

    fn dew_point_as_celsius(&self, relative_humidity: f32) -> Option<f32> {
        if relative_humidity.is_nan()
            || relative_humidity > 100.0
            || relative_humidity < 0.0{
            return None;
        }
        let temperature = self.temperature_as_millicelsius()?;
        Some(utils::dew_point(temperature as f32/1000.0, relative_humidity))
    }

    fn saturation_vapor_pressure_as_hpa(&self) -> Option<f32> {
        utils::saturation_vapor_pressure_as_hpa(self.temperature_as_millicelsius()? as f32 / 1000.0)
    }
}

pub trait TransmitterPower {
    /// Returns transmitter power as dBm if available.
    fn tx_power_as_dbm(&self) -> Option<i8>;
}

pub trait Pm25 {
    /// Returns PM2.5 as mass per volume if available. Unit is 10µg/m^3
    fn pm25_as_10micrograms_per_cubicmeter(&self) -> Option<u16>;
    /// Returns PM2.5 as mass per volume if available. Unit is µg/m^3
    fn pm25_as_micrograms_per_cubicmeter(&self) -> Option<f32> {
        self.pm25_as_10micrograms_per_cubicmeter()
            .map(|v| v as f32 / 10.0)
    }
}

pub trait Co2 {
    /// Returns CO2 as parts per million if available.
    fn co2_as_ppm(&self) -> Option<u16>;
}

pub trait Voc {
    /// VOC index, unitless
    fn voc_index(&self) -> Option<u16>;
}

pub trait Nox {
    fn nox_index(&self) -> Option<u16>;
}

pub trait Lux {
    fn lux_as_logarithmic_value(&self) -> Option<u8>;
    #[allow(dead_code)]
    fn lux_as_normalized_value(&self) -> Option<f32> {
        self.lux_as_logarithmic_value().and_then(|v| {
            let max_code = 254;
            if v == max_code + 1 {
                return None;
            } else {
                let max_value: f32 = 65535.0;
                let delta = (max_value + 1.0).ln() / f32::from(max_code);
                let mut value = ((v as f32 * delta).exp() - 1.0) * 100.0;
                value = value.round() / 100.0;
                Some(value)
            }
        })
    }
}

pub trait DataFormat {
    fn get_dataformat(&self) -> Option<u8>;
}

pub trait AirDensity: Temperature + Pressure + Humidity {
    fn get_air_density_grams_per_cubic_meter(&self) -> Option<u16>;
    fn get_air_density_kg_per_m3(&self) -> Option<f32> {
        self.get_air_density_grams_per_cubic_meter().map(|v| v as f32 / 1000.0)
    }
}

pub trait ProtocolPayload:
Acceleration
+ BatteryPotential
+ Humidity
+ MacAddress
+ MeasurementSequenceNumber
+ MovementCounter
+ Pressure
+ Temperature
+ TransmitterPower
+ Pm25
+ Co2
+ Voc
+ Nox
+ Lux
+ DataFormat
+ AirDensity
{
    const VERSION: u8;
    const SIZE: usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Value {
        temperature: Option<u32>,
        lux: Option<u8>,
    }

    impl Temperature for Value {
        fn temperature_as_millikelvins(&self) -> Option<u32> {
            self.temperature
        }
    }

    impl Lux for Value {
        fn lux_as_logarithmic_value(&self) -> Option<u8> {
            self.lux
        }
    }

    macro_rules! test_lux_logarithmic_conversions {
        (
            $(
                test $name: ident {
                    lux_log: $lux_log: expr,
                    lux_normalized: $lux_normalized: expr,
                }
            )+
        ) => {
            $(
                #[test]
                fn $name() {
                    let value = Value {
                        lux: $lux_log,
                        temperature: None,
                    };
                    assert_eq!(value.lux_as_normalized_value(), $lux_normalized);
                }
            )+
        };
    }

    test_lux_logarithmic_conversions! {
        test zero_lux {
            lux_log: Some(0),
            lux_normalized: Some(0f32),
        }

        test one_lux {
            lux_log: Some(1),
            lux_normalized: Some(0.04),
        }

        test ten_lux {
            lux_log: Some(0x10),
            lux_normalized: Some(1.01),
        }

        test eighty_lux {
            // For some reason this differs from ruuvis examples by two (in ruuvis examples this is
            // 0x80, but with 0x7E the calculated value is to the second decimal exactly the same
            // value)
            lux_log: Some(0x7E),
            lux_normalized: Some(244.06),
        }

        test from_valid_values_lux {
            // This value is from the example showing valid values. Differs by one decimal
            // (in example the value is 13 026.67)
            lux_log: Some(0xD9),
            lux_normalized: Some(13_026.68),
        }

        test max_lux {
            lux_log: Some(0xFE),
            lux_normalized: Some(65535.00),
        }
    }

    macro_rules! test_kelvins_to_celsius_conversions {
        (
            $(
                test $name: ident {
                    millikelvins: $millikelvins: expr,
                    millicelsius: $millicelsius: expr,
                }
            )+
        ) => {
            $(
                #[test]
                fn $name() {
                    let value = Value {
                        lux: None,
                        temperature: $millikelvins,
                    };
                    assert_eq!(value.temperature_as_millicelsius(), $millicelsius);
                }
            )+
        };
    }

    test_kelvins_to_celsius_conversions! {
        test zero_kelvins {
            millikelvins: Some(0),
            millicelsius: Some(-273_150),
        }

        test zero_celsius {
            millikelvins: Some(273_150),
            millicelsius: Some(0),
        }

        test sub_zero_celsius_1 {
            millikelvins: Some(263_080),
            millicelsius: Some(-10_070),
        }

        test sub_zero_celsius_2 {
            millikelvins: Some(194_924),
            millicelsius: Some(-78_226),
        }

        test above_zero_celsius_1 {
            millikelvins: Some(4_343_934),
            millicelsius: Some(4_070_784),
        }

        test above_zero_celsius_2 {
            millikelvins: Some(291_655),
            millicelsius: Some(18_505),
        }

        test no_temperature {
            millikelvins: None,
            millicelsius: None,
        }
    }
}
