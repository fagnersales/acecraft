use toml::Value;

pub fn to_coords(value: &Value) -> [f64; 3] {
    let coords = value
        .as_array()
        .unwrap()
        .iter()
        .map(|coord| coord.as_float().unwrap())
        .collect::<Vec<f64>>();

    assert_eq!(coords.len(), 3);

    coords.try_into().unwrap()
}
