//! Seed the database with mock data for development / testing.
//!
//! Usage (from the `backend/` directory):
//!   cargo run --bin seed
//!
//! The DATABASE_URL env var (or .env file) is used to locate the database.
//! The script is idempotent: re-running it will not insert duplicate rows.

use sqlx::SqlitePool;

// ---------------------------------------------------------------------------
// Static mock data
// ---------------------------------------------------------------------------

struct MockUser {
    oid: &'static str,
    email: &'static str,
    display_name: &'static str,
    full_name: &'static str,
    age: i64,
    is_rso: bool,
    major: &'static str,
    bio: &'static str,
    interests: &'static [&'static str],
}

const MOCK_USERS: &[MockUser] = &[
    MockUser {
        oid: "seed-oid-001",
        email: "alice@university.edu",
        display_name: "alice",
        full_name: "Alice Nguyen",
        age: 20,
        is_rso: false,
        major: "Computer Science",
        bio: "Coffee-fuelled coder who loves hiking and terrible puns.",
        interests: &["Hiking", "Coffee", "Video Games", "Open Source"],
    },
    MockUser {
        oid: "seed-oid-002",
        email: "bob@university.edu",
        display_name: "bob",
        full_name: "Bob Martinez",
        age: 22,
        is_rso: true,
        major: "Mechanical Engineering",
        bio: "Robotics club president. Can fix (and break) almost anything.",
        interests: &["Robotics", "3D Printing", "Cycling", "Coffee"],
    },
    MockUser {
        oid: "seed-oid-003",
        email: "carol@university.edu",
        display_name: "carol",
        full_name: "Carol Zhang",
        age: 21,
        is_rso: false,
        major: "Biology",
        bio: "Pre-med student, amateur photographer, and plant parent.",
        interests: &["Photography", "Hiking", "Cooking", "Reading"],
    },
    MockUser {
        oid: "seed-oid-004",
        email: "david@university.edu",
        display_name: "david",
        full_name: "David Osei",
        age: 23,
        is_rso: true,
        major: "Economics",
        bio: "Aspiring economist who moonlights as a jazz drummer.",
        interests: &["Jazz", "Economics", "Chess", "Cycling"],
    },
    MockUser {
        oid: "seed-oid-005",
        email: "eve@university.edu",
        display_name: "eve",
        full_name: "Eve Kowalski",
        age: 19,
        is_rso: false,
        major: "Psychology",
        bio: "Bookworm and aspiring therapist. Ask me about cognitive biases.",
        interests: &["Reading", "Psychology", "Yoga", "Cooking"],
    },
    MockUser {
        oid: "seed-oid-006",
        email: "frank@university.edu",
        display_name: "frank",
        full_name: "Frank Delacroix",
        age: 24,
        is_rso: false,
        major: "Fine Arts",
        bio: "Painter, illustrator, and chronic overthinker.",
        interests: &["Painting", "Photography", "Chess", "Video Games"],
    },
    MockUser {
        oid: "seed-oid-007",
        email: "grace@university.edu",
        display_name: "grace",
        full_name: "Grace Kim",
        age: 20,
        is_rso: true,
        major: "Data Science",
        bio: "Numbers person by day, K-drama binge-watcher by night.",
        interests: &["Open Source", "Yoga", "Cooking", "Chess"],
    },
    MockUser {
        oid: "seed-oid-008",
        email: "henry@university.edu",
        display_name: "henry",
        full_name: "Henry Okafor",
        age: 22,
        is_rso: false,
        major: "Physics",
        bio: "Astrophysics enthusiast who also happens to love reggae music.",
        interests: &["Jazz", "Cycling", "3D Printing", "Reading"],
    },
];

// Pairs (liker_index, liked_index, is_like) — zero-based into MOCK_USERS.
// Mutual likes between (0,1), (2,3), (4,5), (6,7) → will become matches.
const MOCK_LIKES: &[(usize, usize, bool)] = &[
    (0, 1, true),
    (1, 0, true), // mutual → match
    (2, 3, true),
    (3, 2, true), // mutual → match
    (4, 5, true),
    (5, 4, true), // mutual → match
    (6, 7, true),
    (7, 6, true), // mutual → match
    (0, 2, true),
    (1, 3, false), // pass
    (3, 5, true),
    (2, 6, false), // pass
];

struct MockMessage {
    sender_idx: usize,
    recipient_idx: usize,
    content: &'static str,
}

const MOCK_MESSAGES: &[MockMessage] = &[
    MockMessage { sender_idx: 0, recipient_idx: 1, content: "Hey! I saw you're in robotics — that's awesome." },
    MockMessage { sender_idx: 1, recipient_idx: 0, content: "Yeah! And I heard you're into open source. Any cool projects lately?" },
    MockMessage { sender_idx: 0, recipient_idx: 1, content: "Working on a Rust CLI tool, nothing fancy yet 😄" },
    MockMessage { sender_idx: 1, recipient_idx: 0, content: "Nice, Rust is fire. We should grab coffee sometime!" },
    MockMessage { sender_idx: 2, recipient_idx: 3, content: "Hi David! Big fan of jazz myself." },
    MockMessage { sender_idx: 3, recipient_idx: 2, content: "Oh really? Have you been to the Thursday night sessions on campus?" },
    MockMessage { sender_idx: 2, recipient_idx: 3, content: "Not yet — would love to go! When's the next one?" },
    MockMessage { sender_idx: 4, recipient_idx: 5, content: "Your paintings look incredible from your profile!" },
    MockMessage { sender_idx: 5, recipient_idx: 4, content: "Thank you! I've been doing more portraits lately. Love your yoga posts btw." },
    MockMessage { sender_idx: 6, recipient_idx: 7, content: "Fellow cyclist here 🚴 Do you ride the river trail?" },
    MockMessage { sender_idx: 7, recipient_idx: 6, content: "Every Sunday morning! The sunrise there is unreal." },
    MockMessage { sender_idx: 6, recipient_idx: 7, content: "I'm in for this Sunday then 🌅" },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn ensure_interest(pool: &SqlitePool, name: &str) -> anyhow::Result<i64> {
    // Insert if it doesn't exist yet, then return the id.
    sqlx::query!(
        "INSERT OR IGNORE INTO interests (name) VALUES (?)",
        name
    )
    .execute(pool)
    .await?;

    let row = sqlx::query!("SELECT id FROM interests WHERE name = ?", name)
        .fetch_one(pool)
        .await?;

    Ok(row.id.expect("interest id should not be null"))
}

async fn upsert_user(pool: &SqlitePool, u: &MockUser) -> anyhow::Result<i64> {
    let is_rso_i: i64 = if u.is_rso { 1 } else { 0 };

    sqlx::query!(
        r#"
        INSERT INTO users (oid, email, display_name, full_name, age, is_rso, major, bio)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(oid) DO UPDATE SET
            email        = excluded.email,
            display_name = excluded.display_name,
            full_name    = excluded.full_name,
            age          = excluded.age,
            is_rso       = excluded.is_rso,
            major        = excluded.major,
            bio          = excluded.bio,
            updated_at   = datetime('now')
        "#,
        u.oid,
        u.email,
        u.display_name,
        u.full_name,
        u.age,
        is_rso_i,
        u.major,
        u.bio,
    )
    .execute(pool)
    .await?;

    let row = sqlx::query!("SELECT id FROM users WHERE oid = ?", u.oid)
        .fetch_one(pool)
        .await?;

    Ok(row.id.expect("user id should not be null"))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:app.db".into());

    println!("Connecting to {database_url} ...");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await?;

    // Run migrations so the schema is up to date even on a fresh DB.
    sqlx::migrate!("./migrations").run(&pool).await?;

    // ------------------------------------------------------------------
    // 1. Users + interests
    // ------------------------------------------------------------------
    let mut user_ids: Vec<i64> = Vec::with_capacity(MOCK_USERS.len());

    for mock in MOCK_USERS {
        let uid = upsert_user(&pool, mock).await?;
        user_ids.push(uid);
        println!("  user '{}' → id {uid}", mock.display_name);

        for interest_name in mock.interests {
            let iid = ensure_interest(&pool, interest_name).await?;
            sqlx::query!(
                "INSERT OR IGNORE INTO user_interests (user_id, interest_id) VALUES (?, ?)",
                uid,
                iid
            )
            .execute(&pool)
            .await?;
        }
    }
    println!("✓ Seeded {} users with interests", MOCK_USERS.len());

    // ------------------------------------------------------------------
    // 2. Likes
    // ------------------------------------------------------------------
    for &(liker_idx, liked_idx, is_like) in MOCK_LIKES {
        let liker_id = user_ids[liker_idx];
        let liked_id = user_ids[liked_idx];
        let is_like_i: i64 = if is_like { 1 } else { 0 };

        sqlx::query!(
            r#"
            INSERT INTO likes (liker_id, liked_id, is_like)
            VALUES (?, ?, ?)
            ON CONFLICT(liker_id, liked_id) DO UPDATE SET
                is_like    = excluded.is_like,
                created_at = created_at
            "#,
            liker_id,
            liked_id,
            is_like_i,
        )
        .execute(&pool)
        .await?;
    }
    println!("✓ Seeded {} likes", MOCK_LIKES.len());

    // ------------------------------------------------------------------
    // 3. Matches (derived from mutual likes)
    // ------------------------------------------------------------------
    // For each pair (a, b) where both a→b and b→a are likes, insert a match.
    let mutual_pairs: &[(usize, usize)] = &[(0, 1), (2, 3), (4, 5), (6, 7)];

    for &(a, b) in mutual_pairs {
        let (u1, u2) = {
            let a_id = user_ids[a];
            let b_id = user_ids[b];
            if a_id < b_id { (a_id, b_id) } else { (b_id, a_id) }
        };

        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO matches (user1_id, user2_id)
            VALUES (?, ?)
            "#,
            u1,
            u2,
        )
        .execute(&pool)
        .await?;
    }
    println!("✓ Seeded {} matches", mutual_pairs.len());

    // ------------------------------------------------------------------
    // 4. Messages
    // ------------------------------------------------------------------
    for msg in MOCK_MESSAGES {
        let sender_id = user_ids[msg.sender_idx];
        let recipient_id = user_ids[msg.recipient_idx];

        // Only insert if this exact (sender, recipient, content) combo doesn't
        // exist yet so the script stays idempotent.
        sqlx::query!(
            r#"
            INSERT INTO messages (sender_id, recipient_id, content)
            SELECT ?, ?, ?
            WHERE NOT EXISTS (
                SELECT 1 FROM messages
                WHERE sender_id = ? AND recipient_id = ? AND content = ?
            )
            "#,
            sender_id,
            recipient_id,
            msg.content,
            sender_id,
            recipient_id,
            msg.content,
        )
        .execute(&pool)
        .await?;
    }
    println!("✓ Seeded {} messages", MOCK_MESSAGES.len());

    println!("\nDone! Database seeded successfully.");

    Ok(())
}
