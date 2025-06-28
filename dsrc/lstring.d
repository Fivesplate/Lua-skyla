module lstring;

import std.string;
import std.conv;
import std.hash;
import std.algorithm;
import std.array;
import std.exception;
import core.stdc.string : memcpy;
import core.stdc.stdlib : malloc, free;

/// Represents a Lua string object
struct TString
{
    size_t hash;           // Cached hash of the string
    size_t len;            // Length of string data (excluding null terminator)
    char[] data;           // String data slice (not necessarily null-terminated)

    /// Create a new TString from a D string slice
    static TString create(string s)
    {
        auto t = TString();
        t.len = s.length;
        t.data = s.dup; // duplicate string data to own it
        t.hash = t.computeHash();
        return t;
    }

    /// Compute a hash of the string data (e.g., FNV-1a or djb2)
    size_t computeHash() const
    {
        // Using std.hash's fnvHash here for example
        import std.digest.fnv : fnvHash;
        return fnvHash(data);
    }

    /// Compare two TString objects for equality by hash and content
    bool equals(const TString other) const
    {
        if (this.hash != other.hash) return false;
        if (this.len != other.len) return false;
        return this.data == other.data;
    }
}

/// String table to intern strings (hash table)
struct StringTable
{
    private TString[][] buckets;
    private size_t size;     // number of buckets
    private size_t count;    // number of strings interned

    /// Initialize string table with capacity (number of buckets)
    void init(size_t capacity)
    {
        size = capacity;
        buckets.length = size;
        foreach (ref b; buckets)
            b = null;
        count = 0;
    }

    /// Intern a string, returns existing TString or adds new
    TString intern(string s)
    {
        auto t = TString.create(s);
        auto bucketIndex = t.hash % size;
        auto bucket = buckets[bucketIndex];

        // Search for existing string with same content
        if (bucket !is null)
        {
            foreach (ref str; bucket)
            {
                if (str.equals(t))
                    return str;
            }
        }
        else
        {
            bucket = [];
        }

        // Not found, add new TString
        bucket ~= t;
        buckets[bucketIndex] = bucket;
        ++count;
        return t;
    }

    /// Number of strings interned
    size_t length() const
    {
        return count;
    }
}

/// Hash a Lua string buffer (C string + length) for external uses
size_t luaS_hash(const(char)* str, size_t l, size_t seed = 0)
{
    // Simple FNV-1a hash implementation as example
    enum size_t fnv_offset_basis = 14695981039346656037UL;
    enum size_t fnv_prime = 1099511628211UL;

    size_t h = fnv_offset_basis ^ seed;
    for (size_t i = 0; i < l; ++i)
    {
        h ^= cast(ubyte)str[i];
        h *= fnv_prime;
    }
    return h;
}

/// Compare two Lua strings (TString) for equality
bool luaS_equal(const TString* a, const TString* b)
{
    if (a is b) return true;      // pointer equality
    if (a is null || b is null) return false;
    if (a.len != b.len) return false;
    if (a.hash != b.hash) return false;
    return a.data == b.data;
}

/// Create a new Lua string from C string with length
TString* luaS_new(const(char)* s, size_t len, ref StringTable table)
{
    // Create a D string slice from C string pointer and length
    string slice = s[0 .. len];
    auto interned = table.intern(slice);
    return &interned;
}

/// Initialize the global string table for Lua strings
void luaS_init(ref StringTable table)
{
    table.init(53); // e.g., start with 53 buckets (prime number)
}
module lstring;

import std.string : toStringz, joiner;
import std.array : Appender;
import std.exception : enforce;

/// Concatenate two Lua strings, returning a new interned TString
TString* luaS_concat(const TString* a, const TString* b, ref StringTable table)
{
    enforce(a !is null && b !is null, "Attempt to concatenate null string");

    // Concatenate slices efficiently
    Appender!char buffer;
    buffer.reserve(a.len + b.len);
    buffer.put(a.data);
    buffer.put(b.data);

    // Intern the concatenated string
    return luaS_new(buffer.data.ptr, buffer.data.length, table);
}

/// Extract substring of a Lua string from `start` to `end` (1-based inclusive, Lua style)
TString* luaS_substring(const TString* s, int start, int end, ref StringTable table)
{
    enforce(s !is null, "Substring of null string");

    // Adjust indices to 0-based, handle negative indices (Lua semantics)
    int len = cast(int) s.len;
    if (start < 0) start = len + start + 1;
    if (end < 0) end = len + end + 1;
    if (start < 1) start = 1;
    if (end > len) end = len;
    if (start > end) return luaS_new("", 0, table);

    auto subLen = cast(size_t)(end - start + 1);
    auto ptr = s.data.ptr + start - 1;
    return luaS_new(ptr, subLen, table);
}

/// Compare two Lua strings lexicographically (like strcmp)
int luaS_compare(const TString* a, const TString* b)
{
    enforce(a !is null && b !is null, "Comparison with null string");

    import std.algorithm : cmp;

    size_t minLen = a.len < b.len ? a.len : b.len;
    int res = a.data[0 .. minLen].cmp(b.data[0 .. minLen]);
    if (res != 0)
        return res;
    return cast(int)(a.len) - cast(int)(b.len);
}

/// Check if a Lua string starts with a given prefix
bool luaS_startswith(const TString* s, string prefix)
{
    enforce(s !is null, "startswith null string");
    if (s.len < prefix.length) return false;
    return s.data[0 .. prefix.length] == prefix;
}

/// Format a string using D's formatted string capabilities (basic printf-like)
TString* luaS_format(ref StringTable table, string fmt, Args...)(Args args)
{
    import std.format : formattedWrite;
    import std.array : appender;

    auto buf = appender!char();
    formattedWrite(buf, fmt, args);
    return luaS_new(buf.data.ptr, buf.data.length, table);
}

/// Helper: Convert Lua TString to C-style null-terminated string (allocates new)
const(char)* luaS_tocstring(const TString* s)
{
    enforce(s !is null, "tocstring null string");
    auto cstr = cast(char*) malloc(s.len + 1);
    enforce(cstr !is null, "Memory allocation failed");
    memcpy(cstr, s.data.ptr, s.len);
    cstr[s.len] = '\0';
    return cstr;
}

/// Free a C string previously allocated by luaS_tocstring
void luaS_freecstring(char* cstr)
{
    if (cstr !is null)
        free(cast(void*) cstr);
}
