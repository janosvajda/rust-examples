// Lesson 1: traits.
//
// A trait is a set of methods that a type promises to have. It describes
// what a type can DO, without saying what it IS. Many different types can
// implement the same trait, and code can then work with any of them.

// ---- Defining a trait ---------------------------------------------------------------

/// Anything that can be summarised in one line for a news feed.
trait Summary {
    /// A required method: every implementor must write it.
    fn author(&self) -> String;

    /// A default method: implementors get it for free, but may override it.
    /// Default methods can call the required ones.
    fn summarise(&self) -> String {
        format!("(Read more from {}…)", self.author())
    }
}

// ---- Implementing it for different types -------------------------------------------

struct Article {
    title: String,
    author: String,
    words: u32,
}

struct Post {
    username: String,
    text: String,
    likes: u32,
}

impl Summary for Article {
    fn author(&self) -> String {
        self.author.clone()
    }

    /// Overrides the default with something more useful for articles.
    fn summarise(&self) -> String {
        format!("{}, by {} ({} min read)", self.title, self.author, self.words / 200)
    }
}

impl Summary for Post {
    fn author(&self) -> String {
        format!("@{}", self.username)
    }
    // No `summarise` here: Post uses the default.
}

// A trait can be implemented for types you didn't write, too, as long as the
// trait itself is yours (lesson 6 explains the exact rule).
impl Summary for String {
    fn author(&self) -> String {
        String::from("anonymous")
    }
    fn summarise(&self) -> String {
        format!("\"{self}\"")
    }
}

// ---- Using a trait: accept any type that implements it ------------------------------

/// `&impl Summary` means "a reference to any type that implements Summary".
fn print_feed_item(item: &impl Summary) {
    println!("    {}", item.summarise());
}

/// The same thing in the longer "trait bound" syntax. Lesson 2 explains it.
fn notify<T: Summary>(item: &T) -> String {
    format!("Breaking news! {}", item.summarise())
}

/// Returning `impl Summary`: the caller knows it gets "something that
/// implements Summary", and the function decides which concrete type.
fn featured_post() -> impl Summary {
    Post {
        username: String::from("ferris"),
        text: String::from("Traits are just promises."),
        likes: 42,
    }
}

// ---- derive: the compiler writes common trait implementations for you ---------------

/// `#[derive(...)]` generates implementations of standard traits. Here:
/// Debug ({:?} printing), Clone (.clone()), PartialEq (==).
#[derive(Debug, Clone, PartialEq)]
struct Tag {
    name: String,
}

fn main() {
    let article = Article {
        title: String::from("Why Rust has no null"),
        author: String::from("Grace"),
        words: 1200,
    };
    let post = Post {
        username: String::from("ana"),
        text: String::from("Just learned about traits!"),
        likes: 3,
    };

    println!("1. Each type implements the trait its own way");
    print_feed_item(&article); // overridden summarise
    print_feed_item(&post); // default summarise
    print_feed_item(&String::from("a plain String in the feed"));

    println!("\n2. The same function works with any implementor");
    println!("    {}", notify(&article));
    println!("    {}", notify(&post));

    println!("\n3. Returning `impl Trait`");
    let featured = featured_post();
    println!("    featured: {}", featured.summarise());
    println!("    post has {} likes and says: {}", post.likes, post.text);

    println!("\n4. derive");
    let tag = Tag { name: String::from("rust") };
    let copy = tag.clone();
    println!("    {tag:?} == {copy:?}: {}", tag == copy);

    // Forgetting a required method:
    //     error[E0046]: not all trait items implemented, missing: `author`
    // Calling a function with a type that doesn't implement the trait:
    //     error[E0277]: the trait bound `Tag: Summary` is not satisfied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overridden_method_is_used() {
        let a = Article { title: "T".into(), author: "A".into(), words: 400 };
        assert_eq!(a.summarise(), "T, by A (2 min read)");
    }

    #[test]
    fn default_method_calls_the_required_one() {
        let p = Post { username: "x".into(), text: String::new(), likes: 0 };
        assert_eq!(p.summarise(), "(Read more from @x…)");
    }

    #[test]
    fn trait_on_a_standard_type() {
        assert_eq!(String::from("hi").summarise(), "\"hi\"");
    }

    #[test]
    fn generic_function_accepts_any_implementor() {
        assert!(notify(&String::from("x")).starts_with("Breaking news!"));
        assert!(notify(&featured_post()).contains("@ferris"));
    }

    #[test]
    fn derived_traits_work() {
        let a = Tag { name: "a".into() };
        assert_eq!(a.clone(), a);
        assert_eq!(format!("{a:?}"), "Tag { name: \"a\" }");
    }
}
