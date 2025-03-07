//! Parameter system for procedural generation.

use super::math::*;
use super::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;

#[derive(Clone, Copy)]
pub enum ParameterKind {
    Categorical,
    Ordered,
}

/// Dna parameter. These are recorded for interactive purposes.
#[derive(Clone)]
pub struct Parameter {
    kind: ParameterKind,
    name: String,
    value: String,
    address: Vec<u32>,
    maximum: u32,
    raw: u32,
    hash: u64,
    choices: Vec<String>,
}

impl Parameter {
    pub fn new(
        kind: ParameterKind,
        name: String,
        value: String,
        address: Vec<u32>,
        maximum: u32,
        raw: u32,
        hash: u64,
        choices: Vec<String>,
    ) -> Self {
        Self {
            kind,
            name,
            value,
            address,
            maximum,
            raw,
            hash,
            choices,
        }
    }
    pub fn kind(&self) -> ParameterKind {
        self.kind
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn value(&self) -> &String {
        &self.value
    }
    pub fn address(&self) -> &Vec<u32> {
        &self.address
    }
    pub fn maximum(&self) -> u32 {
        self.maximum
    }
    pub fn maximum_f32(&self) -> f32 {
        self.maximum as f32
    }
    pub fn raw(&self) -> u32 {
        self.raw
    }
    pub fn hash(&self) -> u64 {
        self.hash
    }
    pub fn choices(&self) -> &Vec<String> {
        &self.choices
    }
}

const ADDRESS_LEVELS: usize = 8;

/// The Dna object contains the necessary, mutable
/// context that is threaded through the generation process.
/// Procedural generator parameter sets are tree shaped.
/// The identity for each parameter is hashed from a local tree address and parameter name.
/// Potential collisions are ignored.
/// We keep the current address inside Dna and update it as parameters are drawn.
#[derive(Clone)]
pub struct Dna {
    /// Current tree address.
    /// When we draw a parameter, we increase the address of the bottommost node by 1.
    /// When we descend in the tree, we add a new level.
    address: Vec<u32>,
    /// Values of drawn parameters.
    genome: HashMap<u64, u32>,
    /// Randomness source.
    rnd: Rnd,
    /// Parameters are recorded in interactive mode.
    interactive: bool,
    /// Drawn parameters for interactive display and editing.
    parameters: Vec<Parameter>,
}

impl Dna {
    /// Create a new Dna from u64 seed.
    pub fn new(seed: u64) -> Dna {
        let rnd = Rnd::from_u64(seed);
        Dna {
            address: vec![0],
            genome: HashMap::new(),
            rnd,
            interactive: true,
            parameters: Vec::new(),
        }
    }

    /// Set the value of a gene.
    pub fn set_value(&mut self, hash: u64, value: u32) {
        self.genome.insert(hash, value);
    }

    /// Parameters are recorded in interactive mode.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Set interactive mode. Parameters are recorded when on (the default).
    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    /// Parameter accessor.
    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    /// Attempt to load Dna from the path.
    pub fn load(path: &std::path::Path) -> Option<(String, Dna)> {
        let mut dna = Dna::new(Rnd::from_time().u64());
        let mut is_first_line = true;
        let mut preamble: String = String::new();
        if let Ok(markup) = std::fs::read_to_string(path) {
            for x in markup.lines() {
                if is_first_line {
                    preamble.push_str(x);
                    is_first_line = false;
                } else if let Some(i) = x.find(' ') {
                    let key = x[..i].parse();
                    let value = x[i + 1..].parse();
                    match (key, value) {
                        (Ok(key), Ok(value)) => {
                            dna.genome.insert(key, value);
                        }
                        _ => return None,
                    }
                }
            }
        }
        Some((preamble, dna))
    }

    pub fn save(&self, path: &std::path::Path, preamble: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(preamble.as_bytes())?;
        for (key, value) in self.genome.iter() {
            file.write_all(format!("{} {}\n", key, value).as_bytes())?;
        }
        Ok(())
    }

    /// Mutate the source Dna.
    pub fn mutate(source: &Dna, seed: u64, mutation_p: f32) -> Dna {
        let mut rnd = Rnd::from_u64(seed);
        let mut dna = Dna::new(rnd.u64());
        for (parameter_hash, source_value) in source.genome.iter() {
            if rnd.f32() >= mutation_p {
                dna.set_value(*parameter_hash, *source_value);
            }
        }
        dna
    }

    /// Finetune the source Dna by only modifying non-structural parameters.
    /// Requires interactive mode.
    pub fn finetune(source: &Dna, seed: u64, mutation_p: f32) -> Dna {
        assert!(source.is_interactive());
        let mut rnd = Rnd::from_u64(seed);
        let mut dna = Dna::new(rnd.u64());
        for parameter in source.parameters() {
            if !parameter.choices().is_empty() || rnd.f32() >= mutation_p {
                dna.set_value(parameter.hash(), parameter.raw());
            }
        }
        dna
    }

    /// Add a parameter.
    fn add_parameter(
        &mut self,
        kind: ParameterKind,
        name: String,
        value: String,
        address: Vec<u32>,
        maximum: u32,
        raw: u32,
        hash: u64,
        choices: Vec<String>,
    ) {
        self.parameters.push(Parameter::new(
            kind, name, value, address, maximum, raw, hash, choices,
        ));
    }

    /// Calculate the current address hash based on our tree location.
    fn get_address_hash(&self) -> u64 {
        let l = self.address.len();
        let n = min(ADDRESS_LEVELS, l);
        let mut hash: u64 = n as u64;
        // Use an ad hoc hash.
        for i in (l - n)..l {
            hash = (hash ^ self.address[i] as u64 ^ (hash >> 32)).wrapping_mul(0xd6e8feb86659fd93);
        }
        (hash ^ (hash >> 32)).wrapping_mul(0xd6e8feb86659fd93)
    }

    /// Calculate a parameter hash based on our tree location and parameter name.
    fn get_parameter_hash(&self, parameter_name: &str) -> u64 {
        let address_hash = self.get_address_hash();
        let mut hasher = DefaultHasher::new();
        parameter_name.hash(&mut hasher);
        hasher.finish() ^ address_hash
    }

    /// Draw a parameter value. Adjusts current tree address.
    /// The value will be added to the genome if it is not there already.
    fn draw_value(&mut self, parameter_hash: u64) -> u32 {
        *self.address.last_mut().unwrap() += 1;
        match self.genome.get(&parameter_hash) {
            Some(value) => *value,
            None => {
                let value = self.rnd.u32();
                self.genome.insert(parameter_hash, value);
                value
            }
        }
    }

    /// Reset the Dna for subsequent generation.
    pub fn reset(&mut self) {
        self.address = vec![0];
        self.parameters.clear();
    }

    /// Returns a full range u32 parameter.
    pub fn u32(&mut self, name: &str) -> u32 {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        if self.is_interactive() {
            self.add_parameter(
                ParameterKind::Categorical,
                name.into(),
                format!("{:?}", value),
                self.address.clone(),
                0xffffffff,
                value,
                hash,
                Vec::new(),
            );
        }
        value
    }

    /// Returns a u32 parameter in the given inclusive range.
    pub fn u32_in(&mut self, name: &str, minimum: u32, maximum: u32) -> u32 {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        let value = value % (maximum - minimum + 1);
        if self.is_interactive() {
            self.add_parameter(
                ParameterKind::Categorical,
                name.into(),
                format!("{:?}", value + minimum),
                self.address.clone(),
                maximum - minimum,
                value,
                hash,
                Vec::new(),
            );
        }
        value + minimum
    }

    /// Returns an f32 parameter in 0...1.
    pub fn f32(&mut self, name: &str) -> f32 {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        let value_f = value as f32 / ((1u64 << 32) as f32);
        if self.is_interactive() {
            self.add_parameter(
                ParameterKind::Ordered,
                name.into(),
                format!("{0:.3}", value_f),
                self.address.clone(),
                0xffffffff,
                value,
                hash,
                Vec::new(),
            );
        }
        value_f
    }

    /// Returns an f32 parameter in minimum...maximum.
    pub fn f32_in(&mut self, name: &str, minimum: f32, maximum: f32) -> f32 {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        let value_f = lerp(minimum, maximum, value as f32 / ((1u64 << 32) as f32));
        if self.is_interactive() {
            self.add_parameter(
                ParameterKind::Ordered,
                name.into(),
                format!("{0:.3}", value_f),
                self.address.clone(),
                0xffffffff,
                value,
                hash,
                Vec::new(),
            );
        }
        value_f
    }

    /// Returns an f32 parameter transformed by the supplied function.
    pub fn f32_xform<T: Fn(f32) -> f32>(&mut self, name: &str, xform: T) -> f32 {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        let value_f = xform(value as f32 / ((1u64 << 32) as f32));
        if self.is_interactive() {
            self.add_parameter(
                ParameterKind::Ordered,
                name.into(),
                format!("{0:.3}", value_f),
                self.address.clone(),
                0xffffffff,
                value,
                hash,
                Vec::new(),
            );
        }
        value_f
    }

    /// Returns the index of a choice.
    pub fn index<const T: usize>(&mut self, name: &str, choices: [(f32, &str); T]) -> u32 {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        let choice_index = if (value as usize) < choices.len() && choices[value as usize].0 > 0.0 {
            value as usize
        } else {
            let total_weight: f32 = choices.iter().map(|(weight, _)| weight).sum();
            let mut value = value as f32 / ((1u64 << 32) as f32) * total_weight;
            let mut choice_index = 0;
            for (i, (weight, _)) in choices.iter().enumerate() {
                value -= weight;
                if value <= 0.0 {
                    choice_index = i;
                    break;
                }
            }
            choice_index
        };
        if self.is_interactive() {
            let mut c = Vec::new();
            for (weight, name) in choices {
                if weight > 0.0 {
                    c.push(name.into());
                }
            }
            self.add_parameter(
                ParameterKind::Categorical,
                name.into(),
                choices[choice_index].1.to_string(),
                self.address.clone(),
                choices.len() as u32 - 1,
                choice_index as u32,
                hash,
                c,
            );
        }
        choice_index as u32
    }

    /// Returns a choice.
    pub fn choice<X: Clone, const T: usize>(
        &mut self,
        name: &str,
        choices: [(f32, &str, X); T],
    ) -> X {
        let hash = self.get_parameter_hash(name);
        let value = self.draw_value(hash);
        let choice_index = if (value as usize) < choices.len() && choices[value as usize].0 > 0.0 {
            value as usize
        } else {
            let total_weight: f32 = choices.iter().map(|(weight, _, _)| weight).sum();
            let mut value = value as f32 / ((1u64 << 32) as f32) * total_weight;
            let mut choice_index = 0;
            for (i, (weight, _, _)) in choices.iter().enumerate() {
                value -= weight;
                if value <= 0.0 {
                    choice_index = i;
                    break;
                }
            }
            choice_index
        };
        if self.is_interactive() {
            let mut c = Vec::new();
            for (weight, name, _) in &choices {
                if *weight > 0.0 {
                    c.push((*name).into());
                }
            }
            self.add_parameter(
                ParameterKind::Categorical,
                name.into(),
                choices[choice_index].1.to_string(),
                self.address.clone(),
                choices.len() as u32 - 1,
                choice_index as u32,
                hash,
                c,
            );
        }
        choices[choice_index].2.clone()
    }

    /// Call a subgenerator.
    pub fn call<X, F: Fn(&mut Dna) -> X>(&mut self, f: F) -> X {
        self.address.push(0);
        let x = f(self);
        self.address.pop();
        *self.address.last_mut().unwrap() += 1;
        x
    }
}
