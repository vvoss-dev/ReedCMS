// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Demo: Testdatenbank anlegen und verwenden

use reedbase::{Database, QueryResult};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ReedBase Testdatenbank Demo\n");

    // 1. Testdatenbank in /tmp anlegen
    let db_path = Path::new("/tmp/reedbase_demo/.reed");
    std::fs::create_dir_all(db_path)?;
    println!("✓ Verzeichnis angelegt: {:?}", db_path);

    // 2. Registry initialisieren (für Versionierung)
    reedbase::registry::init_registry(db_path)?;
    reedbase::registry::set_base_path(db_path.to_path_buf());
    println!("✓ Registry initialisiert");

    // 3. Datenbank öffnen
    let db = Database::open(db_path)?;
    println!("✓ Datenbank geöffnet\n");

    // 4. Tabelle erstellen
    db.create_table("text", None)?;
    println!("✓ Tabelle 'text' erstellt");

    // 5. Testdaten einfügen
    println!("\n📝 Füge Testdaten ein...");
    let test_data = vec![
        ("page.title@de", "Willkommen", "Deutscher Seitentitel"),
        ("page.title@en", "Welcome", "English page title"),
        ("menu.home@de", "Startseite", "Menüpunkt Startseite"),
        ("menu.home@en", "Home", "Menu item home"),
        ("menu.about@de", "Über uns", "Menüpunkt Über uns"),
        ("menu.about@en", "About us", "Menu item about"),
        (
            "footer.copyright@de",
            "© 2025 Alle Rechte vorbehalten",
            "Copyright-Text",
        ),
        (
            "footer.copyright@en",
            "© 2025 All rights reserved",
            "Copyright text",
        ),
    ];

    for (key, value, description) in &test_data {
        let sql = format!(
            "INSERT INTO text (key, value, description) VALUES ('{}', '{}', '{}')",
            key, value, description
        );
        db.execute(&sql, "demo_user")?;
        println!("  ✓ Eingefügt: {}", key);
    }

    // 6. Abfragen ausführen
    println!("\n🔍 Führe Abfragen aus...\n");

    // Alle deutschen Einträge
    println!("1. Alle deutschen Einträge:");
    let result = db.query("SELECT * FROM text WHERE key LIKE '%@de'")?;
    print_results(&result);

    // Nur Menüeinträge
    println!("\n2. Nur Menüeinträge:");
    let result = db.query("SELECT * FROM text WHERE key LIKE 'menu.%'")?;
    print_results(&result);

    // Sortiert nach Schlüssel
    println!("\n3. Alle Einträge sortiert:");
    let result = db.query("SELECT key, value FROM text ORDER BY key ASC LIMIT 3")?;
    print_results(&result);

    // 7. Index erstellen (wenn noch nicht vorhanden)
    println!("\n⚡ Erstelle Index auf 'key' Spalte...");
    let index_result = db.create_index("text", "key");
    match index_result {
        Ok(_) => println!("✓ Index erstellt"),
        Err(e) => println!("ℹ️ Index existiert bereits: {:?}", e),
    }

    // 8. Statistiken anzeigen
    println!("\n📊 Datenbank-Statistiken:");
    let stats = db.stats();
    println!("  Tabellen:      {}", stats.table_count);
    println!("  Gesamt Zeilen: {}", stats.total_rows);
    println!("  Queries:       {}", stats.query_count);
    println!("  Inserts:       {}", stats.insert_count);
    println!("  Indices:       {}", db.list_indices().len());

    // 9. Dateisystem zeigen
    println!("\n📁 Dateisystem-Struktur:");
    show_directory_tree(db_path)?;

    println!("\n✅ Demo abgeschlossen!");
    println!("💡 Datenbank liegt in: /tmp/reedbase_demo/.reed");
    println!("   Du kannst sie mit 'reedbase' CLI weiter nutzen!");

    Ok(())
}

fn print_results(result: &QueryResult) {
    match result {
        QueryResult::Rows(rows) => {
            println!("  → {} Zeilen gefunden:", rows.len());
            for (i, row) in rows.iter().enumerate().take(5) {
                let key = row.get("key").map_or("?", |v| v.as_str());
                let value = row.get("value").map_or("?", |v| v.as_str());
                println!("    {}. {} = {}", i + 1, key, value);
            }
            if rows.len() > 5 {
                println!("    ... und {} weitere", rows.len() - 5);
            }
        }
        QueryResult::Aggregation(val) => {
            println!("  → Aggregationsergebnis: {}", val);
        }
    }
}

fn show_directory_tree(path: &Path) -> std::io::Result<()> {
    use std::fs;

    println!("  .reed/");

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                if entry.path().is_dir() {
                    println!("    ├── {}/", name.to_string_lossy());
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        for sub in sub_entries {
                            if let Ok(sub) = sub {
                                println!("    │   ├── {}", sub.file_name().to_string_lossy());
                            }
                        }
                    }
                } else {
                    println!("    ├── {}", name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
}
