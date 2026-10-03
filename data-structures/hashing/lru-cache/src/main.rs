use lru_cache::LruCache;

/// Pretends to be a slow lookup, like a database query or a network call.
fn load_profile(user_id: u32) -> String {
    format!("profile of user {user_id}")
}

fn main() {
    let mut cache = LruCache::new(3);
    let (mut hits, mut misses) = (0, 0);

    for user_id in [1, 2, 3, 1, 4, 1, 2, 5, 1] {
        if let Some(profile) = cache.get(&user_id) {
            hits += 1;
            println!("user {user_id}: hit  ({profile})");
        } else {
            misses += 1;
            let evicted = cache.put(user_id, load_profile(user_id));
            match evicted {
                Some((old_id, _)) => println!("user {user_id}: miss, loaded (evicted user {old_id})"),
                None => println!("user {user_id}: miss, loaded"),
            }
        }
        println!("         cache, most recent first: {:?}", cache.keys_by_recency());
    }

    println!("\n{hits} hits, {misses} misses");
}
