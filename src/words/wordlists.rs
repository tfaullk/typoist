/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗     ██╗    ██████╗     ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝    ███║   ██╔═████╗   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║       ╚██║   ██║██╔██║   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║        ██║   ████╔╝██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║        ██║██╗╚██████╔╝██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝        ╚═╝╚═╝ ╚═════╝ ╚═╝ ╚═════╝

Made with ♥ by tfaullk


*/

// the 200 most common english words — nothing exotic, just what you'd
// actually type in
pub const ENGLISH_200: &[&str] = &[
    "the", "be", "of", "and", "a", "to", "in", "he", "have", "it",
    "that", "for", "they", "with", "as", "not", "on", "she", "at", "by",
    "this", "we", "you", "do", "but", "from", "or", "which", "one", "would",
    "all", "will", "there", "say", "who", "make", "when", "can", "more", "if",
    "no", "man", "out", "other", "so", "what", "time", "up", "go", "about",
    "than", "into", "could", "state", "only", "new", "year", "some", "take", "come",
    "these", "know", "see", "use", "get", "like", "then", "first", "any", "work",
    "now", "may", "such", "give", "over", "think", "most", "even", "find", "day",
    "also", "after", "way", "many", "must", "look", "before", "great", "back", "through",
    "long", "where", "much", "should", "well", "people", "down", "own", "just", "because",
    "good", "each", "those", "feel", "seem", "how", "high", "too", "place", "little",
    "world", "very", "still", "nation", "hand", "old", "life", "tell", "write", "become",
    "here", "show", "house", "both", "between", "need", "mean", "call", "develop", "under",
    "last", "right", "move", "thing", "general", "school", "never", "same", "another", "begin",
    "while", "number", "part", "turn", "real", "leave", "might", "want", "point", "form",
    "off", "child", "few", "small", "since", "against", "ask", "late", "home", "interest",
    "large", "person", "end", "open", "public", "follow", "during", "present", "without", "again",
    "hold", "govern", "around", "possible", "head", "consider", "word", "program", "problem", "however",
];

// rust keywords and common api names, since that's what you type all day anyway
pub const CODE_200: &[&str] = &[
    "fn", "let", "mut", "impl", "trait", "struct", "enum", "match", "if", "else",
    "for", "while", "loop", "return", "use", "mod", "pub", "crate", "self", "super",
    "async", "await", "move", "where", "dyn", "Box", "Vec", "String", "Option", "Result",
    "Some", "None", "Ok", "Err", "map", "and_then", "unwrap", "expect", "clone", "iter",
    "into", "from", "as_ref", "borrow", "ownership", "lifetime", "generic", "closure", "macro", "derive",
    "usize", "isize", "u8", "u16", "u32", "u64", "i32", "i64", "f32", "f64",
    "bool", "char", "str", "println", "eprintln", "format", "assert", "panic", "todo", "unimplemented",
    "const", "static", "type", "ref", "in", "break", "continue", "unsafe", "extern", "true",
    "false", "thread", "spawn", "join", "channel", "send", "recv", "lock", "mutex", "arc",
];

// pool the punctuation sets pull from
pub const PUNCTUATION_CHARS: &[char] = &[
    '.', ',', ';', ':', '!', '?', '-', '_', '(', ')', '[', ']', '{', '}',
    '"', '\'', '/', '\\', '@', '#', '$', '%', '&', '*', '+', '=',
];
