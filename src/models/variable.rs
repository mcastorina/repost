
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct Variable {
        name: String,
        env_vals: HashMap<String, String>,
    }

    impl Variable {
        pub fn new(
            name: String,
            env_vals: Vec<(String, String)>,
        ) -> Result<Variable, &'static str> {
            let mut env_vals_map = HashMap::new();
            for env_val in env_vals {
                let (env, val) = env_val;
                if env_vals_map.contains_key(&env) {
                    return Err("multiple values provided for an environment");
                }
                env_vals_map.insert(env, val);
            }
            Ok(Variable {
                name,
                env_vals: env_vals_map,
            })
        }
        pub fn save(&self) -> Result<(), &'static str> {
            Err("save not implemented")
        }
    }
