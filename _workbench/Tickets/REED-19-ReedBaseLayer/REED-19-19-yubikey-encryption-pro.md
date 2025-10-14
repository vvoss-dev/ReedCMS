# REED-19-19: YubiKey Encryption System (Pro Feature)

**Layer**: REED-19 (ReedBase Layer)  
**Status**: Planned  
**Priority**: Medium (Pro Feature)  
**Complexity**: High  
**Estimated Effort**: 10-14 days  

**Dependencies**:
- REED-19-02 (Universal Table API) - MUST be completed first
- REED-19-09 (Column Schema Validation) - MUST be completed first
- REED-19-16 (Database Registry) - SHOULD be completed first

**Related Tickets**:
- REED-19-17 (Multi-Location Sync) - encrypted data sync
- REED-19-12 (ReedQL) - query encrypted columns

---

## Problem Statement

Web applications handling sensitive data need **encryption-at-rest** with:

1. **Hardware-backed security** (not software keys)
2. **Team access management** (multiple users can decrypt)
3. **Selective encryption** (only sensitive columns, not everything)
4. **Compliance-ready** (GDPR, HIPAA, PCI-DSS requirements)
5. **Reasonable performance** (not 100× slower)
6. **Simple key management** (no AWS KMS complexity/cost)

**Current limitations:**
- No encryption support (plaintext only)
- No compliance-ready features
- No team key management

**User expectations:**
```bash
# Admin: Initialize project with YubiKey
rdb project:init "ByVoss CMS"
# → Touch YubiKey to create root key

# Admin: Provision YubiKey for team member
rdb project:provision-yubikey "ByVoss CMS" --member alice@byvoss.dev
# → YubiKey provisioned, hand to Alice

# Alice: Query encrypted data
rdb db:query users "SELECT ssn FROM users WHERE id = 123"
# → Touch YubiKey to decrypt
# → Returns: 123-45-6789
```

---

## Solution Overview

Implement a **YubiKey-based encryption system** as optional Pro feature with:

1. **Admin YubiKey** = Project Root Key (creates project)
2. **Member YubiKeys** = Provisioned by admin, handed out physically
3. **Selective encryption** = Per-column opt-in via schema
4. **Envelope encryption** = Symmetric DEK + Multi-key asymmetric KEKs
5. **Revocation support** = Deactivate members, optional re-encryption
6. **Performance-aware** = Clear messaging about trade-offs

**Key principle:** Pro ≠ Slower, Encryption = Opt-in per column

---

## Architecture

### Core Components

```
┌────────────────────────────────────────────────────────────┐
│ YubiKey Encryption System Architecture                    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Admin YubiKey (Root)                                      │
│       │                                                    │
│       ├─► Creates Project Root Key                        │
│       ├─► Provisions Member YubiKeys                      │
│       └─► Revokes Member Access                           │
│                                                            │
│  Member YubiKey (Alice)                                    │
│       │                                                    │
│       ├─► Signed by Project Root                          │
│       ├─► Decrypts Team Data                              │
│       └─► Private Key in Secure Element (never exported)  │
│                                                            │
│  Encryption Flow:                                          │
│       │                                                    │
│       ▼                                                    │
│  1. Generate DEK (Data Encryption Key)                     │
│  2. Encrypt plaintext with DEK (AES-256-GCM)               │
│  3. Encrypt DEK for EACH active member (RSA-OAEP)          │
│  4. Store: Envelope { ciphertext, iv, deks[] }             │
│                                                            │
│  Decryption Flow:                                          │
│       │                                                    │
│       ▼                                                    │
│  1. Find DEK for user's YubiKey                            │
│  2. Decrypt DEK with YubiKey (touch required)              │
│  3. Decrypt ciphertext with DEK                            │
│  4. Return plaintext                                       │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Data Structures

```rust
/// Project encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEncryption {
    pub project_name: String,
    pub created_at: String,
    pub root_key: RootKey,
    pub members: Vec<MemberKey>,
}

/// Admin's YubiKey (Project Root)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootKey {
    pub yubikey_serial: String,
    pub slot: String,              // PIV slot (typically "9c")
    pub public_key: String,        // PEM-encoded public key
    pub admin_email: String,
    pub created_at: String,
}

/// Team member's YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberKey {
    pub email: String,
    pub yubikey_serial: String,
    pub slot: String,
    pub public_key: String,
    pub provisioned_at: String,
    pub provisioned_by: String,
    pub status: MemberStatus,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberStatus {
    Active,
    Revoked,
}

/// Encrypted envelope (stored in CSV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub algorithm: String,         // "AES-256-GCM"
    pub iv: String,                // Base64-encoded IV
    pub ciphertext: String,        // Base64-encoded encrypted data
    pub deks: Vec<EncryptedDEK>,   // DEKs for each member
}

/// Encrypted Data Encryption Key (for one member)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedDEK {
    pub yubikey_serial: String,
    pub encrypted_dek: String,     // Base64-encoded RSA-encrypted DEK
}
```

---

## Implementation Plan

### Module Structure

```
src/reedcms/reedbase/
├── encryption/
│   ├── mod.rs                    # Module exports
│   ├── yubikey.rs                # YubiKey interface (via yubikey crate)
│   ├── project.rs                # Project initialization
│   ├── provision.rs              # Member YubiKey provisioning
│   ├── envelope.rs               # Envelope encryption/decryption
│   ├── revocation.rs             # Member revocation
│   ├── rotation.rs               # Key rotation & re-encryption
│   └── encryption_test.rs        # Integration tests
```

---

## Detailed Implementation

### 1. YubiKey Interface (`yubikey.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! YubiKey interface for PIV-based encryption.
//!
//! Uses YubiKey PIV (Personal Identity Verification) applet
//! for RSA key generation and decryption operations.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use yubikey::{YubiKey, piv};

/// Detect connected YubiKey
pub fn detect_yubikey() -> ReedResult<YubiKey> {
    let yubikey = YubiKey::open()
        .map_err(|e| ReedError::EncryptionError {
            message: format!("No YubiKey detected: {}", e),
        })?;
    
    Ok(yubikey)
}

/// Generate RSA keypair in YubiKey slot
pub fn generate_keypair(yubikey: &mut YubiKey, slot: piv::SlotId) -> ReedResult<String> {
    println!("Touch YubiKey to generate keypair...");
    
    let public_key = yubikey
        .generate(
            slot,
            piv::AlgorithmId::Rsa2048,
            piv::PinPolicy::Default,
            piv::TouchPolicy::Cached,
        )
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Failed to generate keypair: {}", e),
        })?;
    
    // Export public key as PEM
    let pem = public_key_to_pem(&public_key)?;
    Ok(pem)
}

/// Decrypt data with YubiKey private key
pub fn decrypt_with_yubikey(
    yubikey: &mut YubiKey,
    slot: piv::SlotId,
    ciphertext: &[u8],
) -> ReedResult<Vec<u8>> {
    println!("Touch YubiKey to decrypt...");
    
    let plaintext = yubikey
        .decrypt_data(ciphertext, slot)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("YubiKey decryption failed: {}", e),
        })?;
    
    Ok(plaintext)
}

/// Get YubiKey serial number
pub fn get_serial(yubikey: &YubiKey) -> ReedResult<String> {
    let serial = yubikey.serial()
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Failed to read YubiKey serial: {}", e),
        })?;
    
    Ok(serial.to_string())
}

/// Convert public key to PEM format
fn public_key_to_pem(public_key: &piv::Certificate) -> ReedResult<String> {
    // TODO: Implement PEM conversion
    Ok("PEM_PLACEHOLDER".to_string())
}
```

---

### 2. Project Initialization (`project.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Project encryption initialization with admin YubiKey.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use super::yubikey;

/// Initialize project with admin YubiKey as root key
pub fn init_project_encryption(
    project_name: &str,
    admin_email: &str,
) -> ReedResult<ProjectEncryption> {
    println!("Initializing project encryption: {}", project_name);
    
    // 1. Detect YubiKey
    let mut yubikey = yubikey::detect_yubikey()?;
    let serial = yubikey::get_serial(&yubikey)?;
    
    println!("YubiKey detected: Serial {}", serial);
    
    // 2. Generate keypair in slot 9c (Digital Signature)
    let public_key = yubikey::generate_keypair(
        &mut yubikey,
        yubikey::piv::SlotId::Signature,
    )?;
    
    println!("✓ Project root key created");
    
    // 3. Create project encryption config
    let config = ProjectEncryption {
        project_name: project_name.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        root_key: RootKey {
            yubikey_serial: serial,
            slot: "9c".to_string(),
            public_key,
            admin_email: admin_email.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        members: vec![],
    };
    
    // 4. Save to registry
    save_project_encryption(&config)?;
    
    println!("✓ Project initialized: {}", project_name);
    println!("✓ Admin: {}", admin_email);
    
    Ok(config)
}

/// Save project encryption config to registry
fn save_project_encryption(config: &ProjectEncryption) -> ReedResult<()> {
    let path = format!(
        "{}/.reedbase/projects/{}.toml",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
        config.project_name
    );
    
    let toml = toml::to_string_pretty(config)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Failed to serialize config: {}", e),
        })?;
    
    std::fs::write(&path, toml)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Failed to write config: {}", e),
        })?;
    
    Ok(())
}
```

---

### 3. Member Provisioning (`provision.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Provision YubiKeys for team members.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use super::{yubikey, project};

/// Provision new YubiKey for team member
pub fn provision_member_yubikey(
    project_name: &str,
    member_email: &str,
) -> ReedResult<MemberKey> {
    // 1. Load project config
    let mut config = project::load_project_encryption(project_name)?;
    
    // 2. Verify admin YubiKey
    println!("Insert ADMIN YubiKey (Serial: {})...", config.root_key.yubikey_serial);
    let mut admin_yubikey = yubikey::detect_yubikey()?;
    let admin_serial = yubikey::get_serial(&admin_yubikey)?;
    
    if admin_serial != config.root_key.yubikey_serial {
        return Err(ReedError::EncryptionError {
            message: format!(
                "Wrong YubiKey! Expected admin key {}, got {}",
                config.root_key.yubikey_serial,
                admin_serial
            ),
        });
    }
    
    println!("✓ Admin authenticated");
    println!("\nRemove admin YubiKey");
    println!("Insert NEW YubiKey to provision for {}...", member_email);
    
    // 3. Wait for user to swap YubiKeys
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    // 4. Detect new member YubiKey
    let mut member_yubikey = yubikey::detect_yubikey()?;
    let member_serial = yubikey::get_serial(&member_yubikey)?;
    
    if member_serial == admin_serial {
        return Err(ReedError::EncryptionError {
            message: "Please remove admin YubiKey and insert NEW YubiKey".to_string(),
        });
    }
    
    println!("Detected: YubiKey Serial {}", member_serial);
    println!("Provisioning for: {}", member_email);
    
    // 5. Generate keypair on member YubiKey
    let public_key = yubikey::generate_keypair(
        &mut member_yubikey,
        yubikey::piv::SlotId::Signature,
    )?;
    
    // 6. Create member key entry
    let member_key = MemberKey {
        email: member_email.to_string(),
        yubikey_serial: member_serial,
        slot: "9c".to_string(),
        public_key,
        provisioned_at: chrono::Utc::now().to_rfc3339(),
        provisioned_by: config.root_key.admin_email.clone(),
        status: MemberStatus::Active,
        revoked_at: None,
    };
    
    // 7. Add to project config
    config.members.push(member_key.clone());
    project::save_project_encryption(&config)?;
    
    println!("✓ YubiKey provisioned for {}", member_email);
    println!("✓ Serial: {}", member_key.yubikey_serial);
    println!("\nHand this YubiKey to {}", member_email);
    
    Ok(member_key)
}
```

---

### 4. Envelope Encryption (`envelope.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Envelope encryption for storage-level encrypted columns.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use rsa::{RsaPublicKey, Oaep, sha2::Sha256};

/// Encrypt value with envelope encryption
///
/// ## Process
/// 1. Generate random DEK (Data Encryption Key)
/// 2. Encrypt plaintext with DEK (AES-256-GCM)
/// 3. Encrypt DEK for each active team member (RSA-OAEP)
/// 4. Return envelope with ciphertext + encrypted DEKs
///
/// ## Performance
/// - DEK generation: ~1ms
/// - AES encryption: ~2-3ms
/// - RSA encryption per member: ~1-2ms
/// - Total: ~5-10ms for 3 members
pub fn encrypt_value(
    plaintext: &str,
    project_name: &str,
) -> ReedResult<String> {
    // 1. Load project config
    let config = project::load_project_encryption(project_name)?;
    
    // 2. Generate random DEK
    let dek = Aes256Gcm::generate_key(&mut OsRng);
    
    // 3. Encrypt plaintext with DEK
    let cipher = Aes256Gcm::new(&dek);
    let nonce = Nonce::from_slice(b"unique nonce"); // TODO: Random nonce
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| ReedError::EncryptionError {
            message: format!("AES encryption failed: {}", e),
        })?;
    
    // 4. Encrypt DEK for each active member
    let mut encrypted_deks = Vec::new();
    
    for member in &config.members {
        if member.status != MemberStatus::Active {
            continue; // Skip revoked members
        }
        
        let public_key = parse_public_key(&member.public_key)?;
        let encrypted_dek = encrypt_dek_with_rsa(&dek.as_slice(), &public_key)?;
        
        encrypted_deks.push(EncryptedDEK {
            yubikey_serial: member.yubikey_serial.clone(),
            encrypted_dek: base64::encode(encrypted_dek),
        });
    }
    
    // 5. Build envelope
    let envelope = EncryptedEnvelope {
        algorithm: "AES-256-GCM".to_string(),
        iv: base64::encode(nonce),
        ciphertext: base64::encode(ciphertext),
        deks: encrypted_deks,
    };
    
    // 6. Serialize to JSON
    let json = serde_json::to_string(&envelope)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Failed to serialize envelope: {}", e),
        })?;
    
    Ok(json)
}

/// Decrypt value from envelope
///
/// ## Process
/// 1. Parse envelope JSON
/// 2. Find DEK for current user's YubiKey
/// 3. Decrypt DEK with YubiKey (requires touch)
/// 4. Decrypt ciphertext with DEK
///
/// ## Performance
/// - Parse JSON: <1ms
/// - YubiKey decrypt: ~3-5ms (includes touch)
/// - AES decrypt: ~1-2ms
/// - Total: ~5-10ms
pub fn decrypt_value(encrypted_json: &str) -> ReedResult<String> {
    // 1. Parse envelope
    let envelope: EncryptedEnvelope = serde_json::from_str(encrypted_json)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Invalid envelope format: {}", e),
        })?;
    
    // 2. Detect YubiKey
    let mut yubikey = yubikey::detect_yubikey()?;
    let serial = yubikey::get_serial(&yubikey)?;
    
    // 3. Find DEK for this YubiKey
    let encrypted_dek = envelope.deks.iter()
        .find(|dek| dek.yubikey_serial == serial)
        .ok_or_else(|| ReedError::EncryptionError {
            message: format!(
                "Your YubiKey ({}) doesn't have access to this data",
                serial
            ),
        })?;
    
    // 4. Decrypt DEK with YubiKey
    let dek_ciphertext = base64::decode(&encrypted_dek.encrypted_dek)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Invalid DEK encoding: {}", e),
        })?;
    
    let dek = yubikey::decrypt_with_yubikey(
        &mut yubikey,
        yubikey::piv::SlotId::Signature,
        &dek_ciphertext,
    )?;
    
    // 5. Decrypt ciphertext with DEK
    let cipher = Aes256Gcm::new_from_slice(&dek)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Invalid DEK: {}", e),
        })?;
    
    let nonce = Nonce::from_slice(&base64::decode(&envelope.iv)?);
    let ciphertext = base64::decode(&envelope.ciphertext)?;
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Decryption failed: {}", e),
        })?;
    
    String::from_utf8(plaintext)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("Invalid UTF-8: {}", e),
        })
}

/// Encrypt DEK with RSA public key
fn encrypt_dek_with_rsa(dek: &[u8], public_key: &RsaPublicKey) -> ReedResult<Vec<u8>> {
    let padding = Oaep::new::<Sha256>();
    let encrypted = public_key
        .encrypt(&mut OsRng, padding, dek)
        .map_err(|e| ReedError::EncryptionError {
            message: format!("RSA encryption failed: {}", e),
        })?;
    
    Ok(encrypted)
}

/// Parse PEM-encoded RSA public key
fn parse_public_key(pem: &str) -> ReedResult<RsaPublicKey> {
    // TODO: Implement PEM parsing
    Err(ReedError::EncryptionError {
        message: "PEM parsing not yet implemented".to_string(),
    })
}
```

---

## Schema Integration

```toml
# .reed/schema/users.schema.toml

[table]
name = "users"
encryption_project = "ByVoss CMS"  # Link to project encryption config

# Plaintext (Free performance)
[[columns]]
name = "username"
type = "string"
# No encryption field → Plaintext
# Performance: <100μs

# Plaintext (Free performance, even in Pro!)
[[columns]]
name = "email"
type = "string"
# No encryption field → Plaintext
# Performance: <100μs

# Storage encryption (Pro feature, opt-in)
[[columns]]
name = "ssn"
type = "string"
encryption = "storage"
# Performance: 5-10ms (YubiKey decrypt)
# Requires: Pro license + YubiKey

# Client encryption (Pro feature, opt-in, user-only)
[[columns]]
name = "credit_card"
type = "string"
encryption = "client"
# Performance: Browser-based (WebCrypto)
# Only user can decrypt (not team)
```

---

## CLI Commands

### 1. Initialize Project Encryption

```bash
# Admin: Initialize project with YubiKey
rdb project:init "ByVoss CMS"

# Output:
→ Initializing project encryption: ByVoss CMS
→ YubiKey detected: Serial 12345678
→ Touch YubiKey to generate keypair...
→ [User touches YubiKey]
→ ✓ Project root key created
→ ✓ Project initialized: ByVoss CMS
→ ✓ Admin: vivian@byvoss.dev

Project Details:
  Name: ByVoss CMS
  Root Key: YubiKey #12345678 Slot 9c
  Admin: vivian@byvoss.dev
  Created: 2025-01-14 12:00:00 UTC
```

---

### 2. Provision Member YubiKey

```bash
# Admin: Provision YubiKey for Alice
rdb project:provision-yubikey "ByVoss CMS" --member alice@byvoss.dev

# Output:
→ Insert ADMIN YubiKey (Serial: 12345678)...
→ YubiKey detected: Serial 12345678
→ ✓ Admin authenticated
→ 
→ Remove admin YubiKey
→ Insert NEW YubiKey to provision for alice@byvoss.dev...
→ 
→ Detected: YubiKey Serial 23456789
→ Provisioning for: alice@byvoss.dev
→ Touch YubiKey to generate keypair...
→ [User touches new YubiKey]
→ ✓ YubiKey provisioned for alice@byvoss.dev
→ ✓ Serial: 23456789
→ 
→ Hand this YubiKey to alice@byvoss.dev
```

---

### 3. Join Project (Member)

```bash
# Alice: Register her provisioned YubiKey
rdb project:join "ByVoss CMS" --email alice@byvoss.dev

# Output:
→ YubiKey detected: YubiKey 5 NFC (Serial: 23456789)
→ Verifying project membership...
→ ✓ YubiKey is provisioned for: alice@byvoss.dev
→ ✓ Project: ByVoss CMS
→ 
→ Touch YubiKey to complete registration...
→ [Alice touches YubiKey]
→ 
→ ✓ Registration complete
→ ✓ You can now access ByVoss CMS data
→ 
→ Welcome to ByVoss CMS, Alice!
```

---

### 4. Query Encrypted Data

```bash
# Alice: Query with encrypted column
rdb db:query users "SELECT id, username, ssn FROM users WHERE id = 123"

# Output:
→ YubiKey detected: Serial 23456789
→ Touch YubiKey to decrypt...
→ [Alice touches YubiKey]
→ ✓ Decrypted

id  | username | ssn
123 | alice    | 123-45-6789
```

---

### 5. Revoke Member

```bash
# Admin: Revoke Bob's access
rdb project:revoke "ByVoss CMS" --member bob@byvoss.dev

# Output:
→ Insert ADMIN YubiKey (Serial: 12345678)...
→ Touch YubiKey to authenticate...
→ 
→ ⚠️  WARNING: This will revoke bob@byvoss.dev's access.
→    YubiKey #34567890 will no longer decrypt project data.
→ 
→ Continue? [y/N]: y
→ 
→ Revoking member...
→ ✓ Member revoked: bob@byvoss.dev
→ ✓ YubiKey #34567890 deactivated
→ 
→ Bob can no longer decrypt NEW data.
→ To revoke access to OLD data, run:
→   rdb project:rotate-keys "ByVoss CMS"
```

---

### 6. Key Rotation

```bash
# Admin: Rotate keys (re-encrypt all data without revoked members)
rdb project:rotate-keys "ByVoss CMS"

# Output:
→ Insert ADMIN YubiKey...
→ Touch YubiKey to authenticate...
→ 
→ ⚠️  WARNING: This will re-encrypt ALL project data.
→    Revoked members (bob@byvoss.dev) will lose access to ALL data.
→ 
→ Analyzing database...
→ Found 125,000 encrypted rows
→ Estimated time: ~8 minutes
→ 
→ Continue? [y/N]: y
→ 
→ Re-encrypting data...
→ Progress: [████████████████████] 100% (125,000/125,000)
→ ✓ Re-encryption complete
→ 
→ Summary:
→   Rows re-encrypted: 125,000
→   Active members: 3 (vivian, alice, charlie)
→   Removed members: 1 (bob)
→   Duration: 7m 34s
```

---

## Performance Characteristics

### Without Encryption (Default, even in Pro)

```csv
# .reed/tables/users/current.csv
id|username|email|created_at
1|alice|alice@example.com|2025-01-14
2|bob|bob@example.com|2025-01-14
```

**Performance:**
- Read: <100μs (HashMap lookup)
- Write: <1ms (CSV write)
- Size: ~100 bytes per row

---

### With Encryption (Opt-in, Pro)

```csv
# .reed/tables/users/current.csv
id|username|ssn|created_at
1|alice|{"algorithm":"AES-256-GCM","iv":"abc...","ciphertext":"def...","deks":[{"yubikey_serial":"12345678","encrypted_dek":"ghi..."},{"yubikey_serial":"23456789","encrypted_dek":"jkl..."}]}|2025-01-14
```

**Performance:**
- Read: 5-10ms (YubiKey decrypt + AES decrypt)
- Write: 10-20ms (AES encrypt + Multi-key RSA encrypt)
- Size: ~2-5KB per row (JSON envelope + multiple DEKs)

---

### Comparison: Selective Encryption

```sql
SELECT id, username, email, ssn FROM users WHERE id = 123
```

**Plaintext columns:** username (plaintext), email (plaintext)
**Encrypted columns:** ssn (YubiKey encrypted)

**Total query time:**
- username: <100μs
- email: <100μs
- ssn: 5-10ms
- **Total: ~5-10ms** (dominated by encrypted column)

**vs. PostgreSQL plaintext:** 2-5ms
**Result:** Comparable performance, hardware-backed security! ✅

---

## Testing Strategy

### Unit Tests
- YubiKey detection (mocked hardware)
- Envelope encryption/decryption
- Member provisioning logic
- Revocation logic

### Integration Tests
- End-to-end encryption flow (with real/simulated YubiKey)
- Multi-member encryption (3+ members)
- Revocation + re-encryption
- Performance benchmarks

### Performance Tests
- Encryption overhead measurement
- Decryption overhead measurement
- Multi-member scaling (1, 5, 10, 20 members)

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Detect YubiKey | <100ms | Time to detect |
| Generate keypair | <3s | Including touch |
| Encrypt value (3 members) | <10ms | DEK gen + AES + 3× RSA |
| Decrypt value | <10ms | YubiKey + AES |
| Provision member | <5s | Including touch |
| Revoke member | <100ms | Status update |
| Rotate keys (10k rows) | <2min | Re-encrypt all |

---

## Documentation Requirements

1. **User Guide**:
   - Setting up project encryption
   - Provisioning YubiKeys for team
   - Daily usage (query encrypted data)
   - Handling lost YubiKeys

2. **Administrator Guide**:
   - YubiKey purchasing recommendations
   - Member onboarding/offboarding
   - Key rotation best practices
   - Compliance documentation (GDPR, HIPAA)

3. **Developer Guide**:
   - Schema configuration (encryption field)
   - Performance characteristics
   - Testing encrypted columns
   - Migration from plaintext

---

## Error Conditions

| Error | Cause | Resolution |
|-------|-------|------------|
| `EncryptionError: No YubiKey detected` | YubiKey not plugged in | Insert YubiKey |
| `EncryptionError: Wrong YubiKey` | Wrong YubiKey inserted | Insert correct YubiKey |
| `EncryptionError: Your YubiKey doesn't have access` | Member not in project | Contact admin for provisioning |
| `EncryptionError: Member revoked` | Access revoked | Contact admin |
| `FeatureError: Pro feature required` | Free version, encryption enabled | Upgrade to Pro |

---

## Dependencies

**External crates**:
- `yubikey = "0.7"` - YubiKey PIV interface
- `rsa = "0.9"` - RSA encryption
- `aes-gcm = "0.10"` - AES-256-GCM encryption
- `base64 = "0.21"` - Base64 encoding
- `serde_json = "1.0"` - JSON serialization

**Internal modules**:
- `reedbase::schema` - Column encryption metadata
- `reedbase::get` - Transparent decryption on read
- `reedbase::set` - Transparent encryption on write
- `reedstream` - Error types

**Hardware dependencies**:
- YubiKey 5 Series (or newer) with PIV support
- USB-A or USB-C port (or NFC for mobile)

---

## Acceptance Criteria

- [ ] Admin can initialize project with YubiKey
- [ ] Admin can provision YubiKeys for team members
- [ ] Members can join project with provisioned YubiKey
- [ ] Schema supports `encryption = "storage"` field
- [ ] Transparent encryption on SET operations
- [ ] Transparent decryption on GET operations (with YubiKey touch)
- [ ] Member revocation implemented
- [ ] Key rotation re-encrypts all data
- [ ] Performance: <10ms decrypt overhead
- [ ] CLI commands implemented and tested
- [ ] All tests pass (unit, integration, performance)
- [ ] Documentation complete (user + admin + developer guides)

---

## Future Enhancements

- **WebAuthn integration**: Browser-based YubiKey access (no CLI)
- **Multi-YubiKey per user**: Backup YubiKey support
- **Automated key rotation**: Scheduled re-encryption
- **Audit logs**: Track who accessed what data when
- **Mobile support**: NFC YubiKey support for iOS/Android
- **Hardware Security Module (HSM)**: Enterprise HSM integration
- **Field-level encryption**: Encrypt parts of a value (e.g., last 4 digits visible)

---

## Pricing Integration

### Free Version
- ❌ YubiKey encryption NOT available
- ✅ All other features (plaintext only)

### Pro Version ($10/month or $100/year)
- ✅ YubiKey encryption available (opt-in per column)
- ✅ Team YubiKey management
- ✅ Member provisioning & revocation
- ✅ Key rotation tools
- ✅ Compliance documentation

**Hardware cost (one-time):**
- YubiKey 5 NFC: ~$50 per member
- Typical team (5 members): ~$250 one-time
- vs. AWS KMS: ~$1-5/month ongoing

---

## Migration Path

### From Free to Pro (Enable Encryption)

```bash
# 1. Upgrade to Pro
rdb project:upgrade "ByVoss CMS" --pro

# 2. Initialize encryption
rdb project:init "ByVoss CMS"

# 3. Update schema (enable encryption for sensitive columns)
# Edit .reed/schema/users.schema.toml:
#   [[columns]]
#   name = "ssn"
#   encryption = "storage"

# 4. Encrypt existing data
rdb db:encrypt users --column ssn

# → Encrypts all existing SSN values with envelope encryption
```

---

## Notes

This is a **Pro feature** that provides enterprise-grade security for web applications handling sensitive data. The key innovation is **selective encryption** - users can encrypt ONLY sensitive columns, maintaining Free-level performance for the rest of their data while achieving compliance for regulated fields.

The YubiKey-based approach provides:
- **Hardware-backed security** (private keys never leave device)
- **Team access management** (multi-member envelope encryption)
- **Reasonable performance** (5-10ms ≈ PostgreSQL plaintext)
- **Simple key management** (physical YubiKey distribution, no cloud key management)
- **Compliance-ready** (GDPR, HIPAA, PCI-DSS)

**Performance positioning:** "Even with encryption, ReedBase Pro is as fast as PostgreSQL plaintext."
