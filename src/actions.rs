use crate::models::UnifiedItem;
use roux::Me;
use roux::util::RouxError;
use std::error::Error;
use std::io::{self, Write};
use csv;
use crate::utils::escape_csv_field;

pub async fn handle_overwrite_action(
    reddit: &Me,
    items: &mut Vec<UnifiedItem>,
    overwrite_text: &str,
    debug_mode: bool,
) -> Result<(), Box<dyn Error>> {
    if debug_mode {
        println!("\n--- Overwriting content for filtered items ---");
    }
    let mut overwrite_success_count = 0;
    let mut overwrite_fail_count = 0;

    for item in items.iter_mut() {
        if debug_mode {
            println!("Attempting to overwrite item ID: {}", item.id);
        }

        let success: bool;
        match reddit.edit(overwrite_text, &item.id).await {
            Ok(_response) => {
                println!("Successfully overwrote {}: {}", item.item_type.to_lowercase(), item.id);
                item.content = overwrite_text.to_string();
                success = true;
            }
            Err(e) => {
                eprintln!("Failed to overwrite {} {}: {}", item.item_type.to_lowercase(), item.id, e);
                if debug_mode { eprintln!("Debug details for RouxError: {:?}", e); }
                success = false;
            }
        }

        if success {
            overwrite_success_count += 1;
        } else {
            overwrite_fail_count += 1;
        }
    }
    if debug_mode || overwrite_success_count > 0 || overwrite_fail_count > 0 {
        println!("\n--- Overwrite Summary ---");
        println!("Successfully overwrote: {} items", overwrite_success_count);
        println!("Failed to overwrite:    {} items", overwrite_fail_count);
    }
    Ok(())
}

pub async fn handle_delete_action(
    reddit: &Me,
    items_to_delete: &[UnifiedItem],
    skip_confirmation: bool,
    debug_mode: bool,
) -> Result<usize, Box<dyn Error>> {
    if items_to_delete.is_empty() {
        if debug_mode {
            println!("No items found to delete based on current filters.");
        }
        return Ok(0);
    }

    let num_items_to_delete = items_to_delete.len();
    println!("\nPreparing to delete {} items.", num_items_to_delete);

    let mut confirmed_to_delete = skip_confirmation;

    if !confirmed_to_delete {
        print!("Are you sure you want to delete these {} items? (yes/No): ", num_items_to_delete);
        io::stdout().flush().map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let mut confirmation_input = String::new();
        io::stdin().read_line(&mut confirmation_input).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        if confirmation_input.trim().to_lowercase() == "yes" {
            confirmed_to_delete = true;
        }
    }

    if confirmed_to_delete {
        println!("Proceeding with deletion...");
        let mut deleted_count = 0;
        let mut failed_count = 0;

        for (index, item) in items_to_delete.iter().enumerate() {
            if debug_mode {
                println!("Deleting item {}/{} (ID: {})...", index + 1, num_items_to_delete, item.id);
            }
            let delete_url = roux::util::url::build_oauth("api/del");
            let params = [("id", item.id.as_str())];
            let token = reddit.config.access_token.as_deref().ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Access token is None after successful login, cannot proceed with deletion."
                )) as Box<dyn Error>
            })?;
            match reddit.client
                .post(&delete_url)
                .bearer_auth(token)
                .form(&params)
                .send()
                .await
                .map_err(RouxError::from)
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let _ = response.text().await; 
                        if debug_mode {
                            println!("Successfully deleted item: {}", item.id);
                        }
                        deleted_count += 1;
                    } else {
                        let status = response.status();
                        let error_body = response.text().await.unwrap_or_else(|e| format!("Could not read error response body: {}", e));
                        eprintln!(
                            "Failed to delete item {} - API Error Status: {}. Details: {}",
                            item.id,
                            status,
                            error_body
                        );
                        failed_count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Error sending delete request for item {}: {}", item.id, e);
                    if debug_mode {
                        eprintln!("Debug details for reqwest error: {:?}", e);
                    }
                    failed_count += 1;
                }
            }
        }
        println!("\n--- Deletion Summary ---");
        println!("  Items targeted for deletion: {}", num_items_to_delete);
        println!("  Successfully deleted:        {}", deleted_count);
        println!("  Failed to delete:            {}", failed_count);
        Ok(deleted_count)
    } else {
        println!("Deletion aborted by user.");
        Ok(0)
    }
}

pub fn handle_csv_export(
    items: &[UnifiedItem],
    file_path: &str,
    debug_mode: bool,
) -> Result<(), Box<dyn Error>> {
    if debug_mode {
        println!("\nExporting {} items to CSV file: {}", items.len(), file_path);
    }
    let mut writer = csv::Writer::from_path(file_path)?;

    writer.write_record([
        "Type",
        "Subreddit",
        "Title",
        "Content",
        "Upvotes",
        "NumComments",
        "Permalink",
        "TimestampUTC",
    ])?;

    for item in items {
        let subreddit_prefix = if item.subreddit.is_empty() { "" } else { "r/" };
        writer.write_record([
            &item.item_type,
            &format!("{}{}", subreddit_prefix, item.subreddit),
            &item.title, 
            &item.content, 
            &item.upvotes.to_string(),
            &item.num_comments.to_string(),
            &format!("https://reddit.com{}", item.permalink),
            &item.created_utc.to_string(),
        ])?;
    }

    writer.flush()?;
    if debug_mode {
        println!("Successfully exported {} items to {}", items.len(), file_path);
    }
    Ok(())
}

pub fn handle_print_to_console(items: &[UnifiedItem], debug_mode: bool) {
    if items.is_empty() {
        if debug_mode {
            println!("No items to output after filtering.");
        }
        return;
    }

    println!("Type,Subreddit,Title,Content,Upvotes,NumComments,Permalink,TimestampUTC");
    for item in items {
        let escaped_title = escape_csv_field(&item.title);
        let escaped_content = escape_csv_field(&item.content);
        let subreddit_prefix = if item.subreddit.is_empty() { "" } else { "r/" };

        println!(
            "\"{}\",\"{}{}\",\"{}\",\"{}\",{},{},\"https://reddit.com{}\",{}",
            item.item_type,
            subreddit_prefix,
            item.subreddit,
            escaped_title,
            escaped_content,
            item.upvotes,
            item.num_comments,
            item.permalink,
            item.created_utc
        );
    }
}
