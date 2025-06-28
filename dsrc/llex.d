module llex;

import std.stdio;
import std.string;
import std.conv;
import std.algorithm;
import core.stdc.string : memcpy;
import core.stdc.stdlib : strtod;

/// Token enumeration (subset)
enum Token
{
    TK_EOS = -1,
    TK_NAME,
    TK_NUMBER,
    TK_STRING,
    TK_FUNCTION,
    TK_IF,
    TK_ELSE,
    TK_FOR,
    TK_WHILE,
    TK_RETURN,
    TK_DO,
    TK_END,
    TK_THEN,
    TK_ELSEIF,
    TK_REPEAT,
    TK_UNTIL,
    TK_BREAK,
    TK_LOCAL,
    TK_GOTO,
    TK_NIL,
    TK_TRUE,
    TK_FALSE,
    TK_SEP,
    TK_OP,
    // add more tokens as necessary
}

/// Lexical state struct to hold lexer state
struct LexState
{
    const(char)* source;    // Source code pointer
    size_t sourceLen;       // Length of source code
    size_t currentPos;      // Current position in source

    int current;            // Current char (int for EOF detection)
    string token;           // Current token lexeme
    Token tokenType;        // Current token type

    int lineNumber;         // Current line number

    // constructor to initialize lexer
    this(const(char)* src, size_t len)
    {
        source = src;
        sourceLen = len;
        currentPos = 0;
        lineNumber = 1;
        nextChar();
    }

    /// Get next char from source or EOF (-1)
    void nextChar()
    {
        if (currentPos >= sourceLen)
        {
            current = -1; // EOF
            return;
        }
        current = source[currentPos];
        ++currentPos;
    }

    /// Skip whitespace characters (space, tab, newline, carriage return)
    void skipWhitespace()
    {
        while (current == ' ' || current == '\t' || current == '\n' || current == '\r')
        {
            if (current == '\n')
                ++lineNumber;
            nextChar();
        }
    }

    /// Read identifier or keyword
    void readName()
    {
        size_t start = currentPos - 1;
        while (isAlphaNum(current) || current == '_')
            nextChar();
        size_t end = currentPos - 1;
        token = cast(string) source[start .. end];

        // Determine if token is a keyword
        tokenType = keywordToken(token);
    }

    /// Read a number literal (integer or float)
    void readNumber()
    {
        size_t start = currentPos - 1;
        bool isFloat = false;

        while (isDigit(current) || current == '.')
        {
            if (current == '.')
                isFloat = true;
            nextChar();
        }

        // Support exponential part
        if (current == 'e' || current == 'E')
        {
            isFloat = true;
            nextChar();
            if (current == '+' || current == '-')
                nextChar();
            while (isDigit(current))
                nextChar();
        }

        size_t end = currentPos - 1;
        token = cast(string) source[start .. end];
        tokenType = Token.TK_NUMBER;
    }

    /// Read a string literal delimited by single or double quotes
    void readString()
    {
        char delimiter = cast(char) current;
        nextChar(); // skip delimiter
        size_t start = currentPos - 1;

        while (current != delimiter && current != -1)
        {
            if (current == '\\')
            {
                nextChar(); // skip escape char
                nextChar();
            }
            else
            {
                nextChar();
            }
        }

        size_t end = currentPos - 1;
        token = cast(string) source[start .. end];
        tokenType = Token.TK_STRING;

        if (current == delimiter)
            nextChar();
        else
            writeln("Warning: unfinished string literal at line ", lineNumber);
    }

    /// Helper: return Token for keywords, else TK_NAME
    Token keywordToken(string s)
    {
        immutable string[] keywords = [
            "and", "break", "do", "else", "elseif", "end", "false",
            "for", "function", "goto", "if", "in", "local", "nil",
            "not", "or", "repeat", "return", "then", "true", "until",
            "while"
        ];

        foreach (idx, kw; keywords)
        {
            if (s == kw)
            {
                static immutable Token[] keywordTokens = [
                    Token.TK_OP, Token.TK_BREAK, Token.TK_DO, Token.TK_ELSE,
                    Token.TK_ELSEIF, Token.TK_END, Token.TK_FALSE,
                    Token.TK_FOR, Token.TK_FUNCTION, Token.TK_GOTO, Token.TK_IF,
                    Token.TK_OP, Token.TK_LOCAL, Token.TK_NIL,
                    Token.TK_OP, Token.TK_REPEAT, Token.TK_RETURN, Token.TK_THEN,
                    Token.TK_TRUE, Token.TK_UNTIL, Token.TK_WHILE
                ];
                return keywordTokens[idx];
            }
        }
        return Token.TK_NAME;
    }

    /// Main lexer function: reads next token
    void nextToken()
    {
        skipWhitespace();

        if (current == -1)
        {
            tokenType = Token.TK_EOS;
            return;
        }

        if (isAlpha(current) || current == '_')
        {
            readName();
            return;
        }

        if (isDigit(current) || (current == '.' && isDigit(peek())))
        {
            readNumber();
            return;
        }

        if (current == '\'' || current == '"')
        {
            readString();
            return;
        }

        // TODO: handle comments, operators, punctuation here

        // For now, read single char token and advance
        token = cast(string) current.to!char;
        tokenType = Token.TK_OP;
        nextChar();
    }

    /// Peek next char without consuming it
    int peek() const
    {
        if (currentPos >= sourceLen)
            return -1;
        return source[currentPos];
    }

    /// Character classification helpers
    bool isAlpha(int c) const { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }
    bool isDigit(int c) const { return c >= '0' && c <= '9'; }
    bool isAlphaNum(int c) const { return isAlpha(c) || isDigit(c); }
}

/**
 * Skip a single line comment starting with `--`
 */
void skipSingleLineComment()
{
    while (current != '\n' && current != -1)
    {
        nextChar();
    }
    // Skip the newline character as well
    if (current == '\n')
    {
        ++lineNumber;
        nextChar();
    }
}

/**
 * Skip a long comment enclosed in --[[ ... ]] or with any number of '=' signs.
 */
void skipLongComment()
{
    int level = skipSeparator();
    if (level < 0)
    {
        // Not a long comment start, treat as normal comment
        skipSingleLineComment();
        return;
    }

    // Skip the opening newline if any
    if (current == '\n')
    {
        ++lineNumber;
        nextChar();
    }

    // Read until matching closing delimiter
    for (;;)
    {
        if (current == -1)
        {
            writeln("Warning: unfinished long comment at line ", lineNumber);
            return;
        }
        else if (current == ']')
        {
            int sep = skipSeparator();
            if (sep == level)
            {
                nextChar();
                return;
            }
        }
        else
        {
            if (current == '\n')
                ++lineNumber;
            nextChar();
        }
    }
}

/**
 * Check for long string/comment delimiter [=*[ and return number of '=' chars.
 * Returns -1 if not a valid delimiter.
 */
int skipSeparator()
{
    // Save state in case it's not a valid delimiter
    size_t savedPos = currentPos;
    int savedCurrent = current;
    int count = 0;
    nextChar();
    while (current == '=')
    {
        nextChar();
        ++count;
    }
    if (current == '[')
    {
        nextChar();
        return count;
    }
    else {
        // Restore state if not a valid delimiter
        currentPos = savedPos;
        current = savedCurrent;
        return -1;
    }
}

/**
 * Read a long string literal, delimited by [=*[ and ]=*].
 * Stores the string content in `token`.
 */
void readLongString()
{
    int level = skipSeparator();
    if (level < 0)
    {
        // Not a long string start, fallback or error
        writeln("Error: invalid long string delimiter at line ", lineNumber);
        return;
    }

    // Skip opening newline if any
    if (current == '\n')
    {
        ++lineNumber;
        nextChar();
    }

    import std.array : Appender;
    Appender!char buffer;
    
    for (;;)
    {
        if (current == -1)
        {
            writeln("Warning: unfinished long string at line ", lineNumber);
            break;
        }
        else if (current == ']')
        {
            int sep = skipSeparator();
            if (sep == level)
            {
                nextChar();
                break;
            }
            else
            {
                buffer.put(']');
                for (int i = 0; i < sep; ++i)
                    buffer.put('=');
            }
        }
        else
        {
            if (current == '\n')
                ++lineNumber;
            buffer.put(cast(char)current);
            nextChar();
        }
    }
    token = buffer.data.to!string;
    tokenType = Token.TK_STRING;
}

/**
 * Handle operator and punctuation tokens.
 */
void readOperator()
{
    // Basic single/double char operators: + - * / % ^ # == ~= <= >= < > = ( ) { } [ ] ; : , . .. ...
    import std.string : startsWith;

    // Peek next character for multi-char operators
    int next_c = peek();

    string op = cast(string) current.to!char;

    // Check double char ops
    if (current == '=' && next_c == '=')
    {
        nextChar();
        nextChar();
        token = "==";
        tokenType = Token.TK_OP;
        return;
    }
    else if (current == '~' && next_c == '=')
    {
        nextChar();
        nextChar();
        token = "~=";
        tokenType = Token.TK_OP;
        return;
    }
    else if (current == '<' && next_c == '=')
    {
        nextChar();
        nextChar();
        token = "<=";
        tokenType = Token.TK_OP;
        return;
    }
    else if (current == '>' && next_c == '=')
    {
        nextChar();
        nextChar();
        token = ">=";
        tokenType = Token.TK_OP;
        return;
    }
    else if (current == '.' && next_c == '.')
    {
        nextChar();
        if (peek() == '.')
        {
            nextChar();
            nextChar();
            token = "...";
            tokenType = Token.TK_OP;
            return;
        }
        token = "..";
        tokenType = Token.TK_OP;
        return;
    }
    else
    {
        // Single char op
        nextChar();
        token = op;
        tokenType = Token.TK_OP;
    }
}

/**
 * Override the main nextToken() function to integrate comment and operator handling.
 */
void nextToken()
{
    skipWhitespace();

    if (current == -1)
    {
        tokenType = Token.TK_EOS;
        return;
    }

    // Handle comments starting with --
    if (current == '-' && peek() == '-')
    {
        nextChar();
        nextChar();
        if (current == '[')
        {
            int sep = skipSeparator();
            if (sep >= 0)
                skipLongComment();
            else
                skipSingleLineComment();
        }
        else
        {
            skipSingleLineComment();
        }
        nextToken(); // after skipping comment, get next token
        return;
    }

    if (isAlpha(current) || current == '_')
    {
        readName();
        return;
    }

    if (isDigit(current) || (current == '.' && isDigit(peek())))
    {
        readNumber();
        return;
    }

    if (current == '\'' || current == '"')
    {
        readString();
        return;
    }

    if (current == '[')
    {
        int sep = skipSeparator();
        if (sep >= 0)
        {
            readLongString();
            return;
        }
        else
        {
            // Single [ token
            token = "[";
            tokenType = Token.TK_OP;
            nextChar();
            return;
        }
    }

    // Handle operators and punctuation
    readOperator();
}