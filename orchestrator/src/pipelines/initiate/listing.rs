use crate::agent_listing::AgentsListing;
use std::fs;
use std::path::Path;

pub fn save_agents_listing<P: AsRef<Path>>(
    listing_path: P,
    listing: &AgentsListing,
) -> std::io::Result<()> {
    let listing_bytes = bincode::serialize(listing).unwrap();
    fs::write(listing_path, listing_bytes)
}
