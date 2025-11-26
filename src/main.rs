mod cli;
mod czkawka;
mod immich;

use base64::{Engine, engine::general_purpose};
use clap::Parser;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io;
use std::sync::Arc;
use std::{cmp::Reverse, error::Error};
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::czkawka::CzkawkaFileEntry;

async fn calculate_file_checksum(file_path: &str) -> io::Result<String> {
    let owned_path = file_path.to_owned();
    let hash = tokio::task::spawn_blocking(move || {
        let mut file = File::open(owned_path)?;
        let mut hasher = Sha1::new();
        io::copy(&mut file, &mut hasher)?;
        Ok::<_, std::io::Error>(general_purpose::STANDARD.encode(hasher.finalize()))
    })
    .await??;

    Ok(hash)
}

async fn process_file_group(
    file_group: &Vec<CzkawkaFileEntry>,
    immich_service: &immich::Service,
    spare_similarity: bool,
    dry_run: bool,
    semaphore: Arc<Semaphore>,
) -> Result<(), Box<dyn Error>> {
    let permit: SemaphorePermit<'_> = semaphore.acquire().await?;

    if !spare_similarity && file_group.iter().any(|f| f.similarity > 0) {
        return Ok(());
    }
    let mut immich_assets: Vec<immich::Asset> = vec![];
    for file in file_group.iter() {
        let checksum = calculate_file_checksum(&file.path).await?;
        let immich_asset = immich_service.get_asset_by_checksum(&checksum).await?;
        immich_assets.push(immich_asset);
    }

    immich_assets.sort_by_key(|a| Reverse(a.created_at));
    let to_delete_assets: Vec<&immich::Asset> = immich_assets.iter().skip(1).collect();
    let to_delete_asset_ids: Vec<&str> = to_delete_assets.iter().map(|f| f.id.as_str()).collect();

    println!(
        "Found {} assets to delete: {:?}",
        to_delete_assets.len(),
        to_delete_assets
    );

    if !dry_run {
        immich_service.delete_assets(to_delete_asset_ids).await?;
    }

    drop(permit);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Cli::parse();
    let immich_service = immich::Service::new(&args.immich_host, &args.immich_api_key);

    let similar_file_groups = czkawka::parse_similar_result(&args.czkawka_output_path)?;
    println!("Number of duplicate groups: {}", similar_file_groups.len());

    let semaphore = Arc::new(Semaphore::new(64));
    let futures: Vec<_> = similar_file_groups
        .iter()
        .map(|file_group| {
            let task_semaphore = Arc::clone(&semaphore);
            process_file_group(
                file_group,
                &immich_service,
                args.spare_similarity,
                args.dry_run,
                task_semaphore,
            )
        })
        .collect();
    let results = futures::future::join_all(futures).await;

    let mut success_count = 0;
    let mut failure_count = 0;
    for result in results {
        match result {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    println!("====================");
    println!("SUMMARY");
    println!("Successfully deduplicated group: {}", success_count);
    println!("Failed deduplicated group: {}", failure_count);

    Ok(())
}
