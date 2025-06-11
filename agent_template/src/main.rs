mod communication;
mod curiosity;
mod fsm;
mod genome;
mod hormones;
mod memory_graph;
mod meta;
mod neural;
mod rl;
mod selfmod;
mod symbolic;

fn main() {
    let genome = genome::Genome::random();
    println!("Agent genome: {:?}", genome);

    // Utilisation de la nouvelle méthode de vérification
    if genome.is_module_active("neural") {
        neural::run();
    }
    if genome.is_module_active("memory_graph") {
        memory_graph::run();
    }
    if genome.is_module_active("symbolic") {
        symbolic::run();
    }
    if genome.is_module_active("curiosity") {
        curiosity::run();
    }
    if genome.is_module_active("fsm") {
        fsm::run();
    }
    if genome.is_module_active("rl") {
        rl::run();
    }
    if genome.is_module_active("hormones") {
        hormones::run();
    }
    if genome.is_module_active("communication") {
        communication::run();
    }
    if genome.is_module_active("selfmod") {
        selfmod::run();
    }
    if genome.is_module_active("meta") {
        meta::run();
    }
}
