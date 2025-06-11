use crate::genome::genes::AgentGenes;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Genome {
    pub genes: AgentGenes,
}

impl Genome {
    pub fn random() -> Self {
        let mut genes = AgentGenes::new(uuid::Uuid::new_v4().to_string());
        let mut rng = rand::thread_rng();

        for config in genes.modules.values_mut() {
            config.active = rng.gen();
        }

        Self { genes }
    }

    pub fn is_module_active(&self, name: &str) -> bool {
        self.genes
            .modules
            .get(name)
            .map(|config| config.active)
            .unwrap_or(false)
    }
}
