use anyhow::{bail, Result};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use clap::Parser;

/// A simple CLI tool to update a Route53 DNS record with the current WAN IP address.
#[derive(Parser, Debug)]
#[command(
    author = "Anton Eicher",
    version = "1.0",
    about = "Updates Route53 DNS records"
)]
struct Args {
    // The AWS profile to use
    #[arg(short, long)]
    profile: String,

    /// The AWS Route53 region
    #[arg(short, long)]
    region: String,

    /// The hosted zone ID for the DNS record (e.g. `/hostedzone/Z31T92M9QYO6S8`)
    #[arg(short='z', long)]
    hosted_zone_id: String,

    /// The DNS record name to update (e.g. `foo.com.`)
    #[arg(short='n', long)]
    record_name: String,

}

async fn get_wan_ip() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://api.ipify.org").await?;
    let ip_address = response.text().await?;
    Ok(ip_address)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Retrieve the current WAN IP address
    let new_ip_address = match get_wan_ip().await {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Failed to retrieve WAN IP address: {}", e);
            bail!("Failed to retrieve WAN IP address");
        }
    };

    println!("Updating DNS record...");
    println!("Hosted Zone ID: {}", args.hosted_zone_id);
    println!("Record Name: {}", args.record_name);
    println!("IP Address: {}", new_ip_address);

    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .profile_name(&args.profile)
        .region(Region::new(args.region))
        .load()
        .await;
    let route53_client = aws_sdk_route53::Client::new(&aws_config);

    // Determine current WAN ip


    // Create a change batch to update the DNS record
    let change_batch = ChangeBatch::builder()
        .changes(
            Change::builder()
                .action(ChangeAction::Upsert)
                .resource_record_set(
                    ResourceRecordSet::builder()
                        .name(&args.record_name)
                        .r#type(RrType::A)
                        .ttl(300)
                        .resource_records(
                            ResourceRecord::builder()
                                .value(&new_ip_address)
                                .build()
                                .expect("build resource record"),
                        )
                        .build()
                        .expect("record set builder"),
                )
                .build()
                .expect("change builder"),
        )
        .build()
        .expect("change batch");

    // Execute change
    route53_client
        .change_resource_record_sets()
        .hosted_zone_id(args.hosted_zone_id)
        .change_batch(change_batch)
        .send()
        .await
        .expect("change resource record sets");

    println!(
        "Successfully changed DNS record for {} to {}",
        args.record_name, new_ip_address
    );
    Ok(())
}
