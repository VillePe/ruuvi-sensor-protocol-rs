const ZERO_CELSIUS_IN_KELVINS: f32 = 273.150;

/// Calculates absolute humidity in g/m3 at the given temperature in Celsius and relative humidity in %
///
/// Formulas from Vaisala Humidity Conversion Formulas PDF
pub fn absolute_humidity_as_grams_per_cubic_meter(
    temperature_celsius: f32,
    relative_humidity: f32,
) -> f32 {
    let es = saturation_vapor_pressure_as_hpa(temperature_celsius).unwrap_or_else(|| {
        panic!(
            "Temperature {} is out of range for humidity calculation",
            temperature_celsius
        )
    });
    let e: f32 = relative_humidity / 100.0 * es;
    (216.7 * e) / (273.15 + temperature_celsius)
}

#[allow(dead_code)]
pub fn dew_point(temperature_celsius: f32, relative_humidity: f32) -> f32 {
    let es = saturation_vapor_pressure_as_hpa(temperature_celsius).unwrap_or_else(|| {
        panic!(
            "Temperature {} is out of range for humidity calculation",
            temperature_celsius
        )
    });
    let e: f32 = relative_humidity / 100.0 * es;
    (243.12 * (e / 6.112).ln()) / (17.62 - (e / 6.112).ln())
}

/// Calculates partial pressure of saturation moisture in hPa at given temperature in Celsius.
pub fn saturation_vapor_pressure_as_hpa(temperature_celsius: f32) -> Option<f32> {
    let vaisala_constants = get_vaisala_constants(temperature_celsius);
    match vaisala_constants {
        None => None,
        Some(constants) => Some(
            constants.A
                * 10f32.powf(
                (constants.m * temperature_celsius) / (constants.Tn + temperature_celsius),
            ),
        ),
    }
}

/// Calculates the air density
///
/// See [Wikipedia link](https://en.wikipedia.org/wiki/Density_of_air#Humid_air)
#[allow(non_snake_case, dead_code)]
pub fn calculate_air_density(temperature_celsius: f32, humidity_percent: f32, pressure_in_pascals: f32) -> f32 {
    // The molar mass of dry air
    let Md = 0.0289652;
    let Mv = 0.018016;
    let R = 8.31446;
    let temp_in_kelvins = ZERO_CELSIUS_IN_KELVINS + temperature_celsius;
    let saturation_vapor_pressure_in_pa = saturation_vapor_pressure_as_hpa(temperature_celsius).unwrap() * 100.0;
    // Pressure of water vapor (Pa)
    let pv = humidity_percent / 100.0 * saturation_vapor_pressure_in_pa;
    // Pressure of dry air (Pa)
    let pd = pressure_in_pascals - pv;
    let result = (pd * Md + pv * Mv) / (R * temp_in_kelvins);
    result
}

fn get_vaisala_constants(temperature_celsius: f32) -> Option<VaisalaConstants> {
    if temperature_celsius < 0.0 {
        Some(VAI_CONSTANTS_MINUS_70_TO_ZERO)
    } else if temperature_celsius < 50.0 {
        Some(VAI_CONSTANTS_MINUS_20_TO_PLUS_50)
    } else if temperature_celsius < 100.0 {
        Some(VAI_CONSTANTS_PLUS_50_TO_PLUS_100)
    } else if temperature_celsius < 150.0 {
        Some(VAI_CONSTANTS_PLUS_100_TO_PLUS_150)
    } else if temperature_celsius < 200.0 {
        Some(VAI_CONSTANTS_PLUS_150_TO_PLUS_200)
    } else if temperature_celsius < 350.0 {
        Some(VAI_CONSTANTS_PLUS_200_TO_PLUS_350)
    } else {
        None
    }
}

#[allow(non_snake_case)]
struct VaisalaConstants {
    A: f32,
    m: f32,
    Tn: f32,
}

const VAI_CONSTANTS_MINUS_20_TO_PLUS_50: VaisalaConstants = VaisalaConstants {
    A: 6.116441,
    m: 7.591386,
    Tn: 240.7263,
};
const VAI_CONSTANTS_PLUS_50_TO_PLUS_100: VaisalaConstants = VaisalaConstants {
    A: 6.004918,
    m: 7.337936,
    Tn: 229.3975,
};
const VAI_CONSTANTS_PLUS_100_TO_PLUS_150: VaisalaConstants = VaisalaConstants {
    A: 5.856548,
    m: 7.27731,
    Tn: 225.1033,
};
const VAI_CONSTANTS_PLUS_150_TO_PLUS_200: VaisalaConstants = VaisalaConstants {
    A: 6.002859,
    m: 7.290361,
    Tn: 227.1704,
};
const VAI_CONSTANTS_PLUS_200_TO_PLUS_350: VaisalaConstants = VaisalaConstants {
    A: 9.980622,
    m: 7.388931,
    Tn: 263.1239,
};
#[allow(dead_code)]
const VAI_CONSTANTS_ZERO_TO_PLUS_200: VaisalaConstants = VaisalaConstants {
    A: 6.089613,
    m: 7.33502,
    Tn: 230.3921,
};
const VAI_CONSTANTS_MINUS_70_TO_ZERO: VaisalaConstants = VaisalaConstants {
    A: 6.114742,
    m: 9.9778707,
    Tn: 273.1466,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::floating_value_test;

    floating_value_test! {
        t_dew_point,
        dew_point(40.0, 50.0),
        27.6,
        0.1
    }

    floating_value_test! {
        t_dew_point_2,
        dew_point(24.3, 53.49),
        14.2,
        0.1
    }

    floating_value_test! {
        t_saturation_vapor_pressure_40c,
        saturation_vapor_pressure_as_hpa(40.0).unwrap()*0.5,
        36.88,
        0.05
    }

    floating_value_test! {
        t_saturation_vapor_pressure_20c,
        saturation_vapor_pressure_as_hpa(20.0).unwrap()*0.8,
        18.7,
        0.05
    }

    floating_value_test! {
        t_absolute_humidity,
        absolute_humidity_as_grams_per_cubic_meter(20.0, 80.0),
        13.82,
        0.05
    }

    floating_value_test! {
        t_absolute_humidity_2,
        absolute_humidity_as_grams_per_cubic_meter(24.3, 53.49),
        11.845,
        0.05
    }

    floating_value_test! {
        t_air_density_standard,
        calculate_air_density(15.0, 00.0, 101_325.0),
        1.225,
        0.0001
    }

    floating_value_test! {
        t_air_density_humidity_100,
        calculate_air_density(15.0, 100.0, 101_325.0),
        1.217,
        0.001
    }
}
