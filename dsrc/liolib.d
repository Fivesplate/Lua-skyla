module liolib;

import std.stdio : File, stdin, stdout, stderr;
import std.file : FileMode;
import std.exception : enforce;
import std.conv : to;
import std.string : strip;

/// Abstract Lua file userdata representation
struct LuaFile
{
    File* file;
    bool isClosed;

    /// Open a file with given mode ("r", "w", "a", etc.)
    static LuaFile openFile(string filename, string mode)
    {
        auto luaFile = LuaFile();
        luaFile.isClosed = false;

        FileMode fileMode;
        final switch (mode)
        {
            case "r": fileMode = FileMode.read; break;
            case "w": fileMode = FileMode.write; break;
            case "a": fileMode = FileMode.append; break;
            case "r+": fileMode = FileMode.update; break;
            case "w+": fileMode = FileMode.updateTruncate; break;
            case "a+": fileMode = FileMode.updateAppend; break;
            default:
                enforce(false, "Invalid file mode: " ~ mode);
        }

        luaFile.file = new File(filename, fileMode);
        return luaFile;
    }

    /// Close the file if open
    void close()
    {
        if (!isClosed && file !is null)
        {
            file.close();
            isClosed = true;
        }
    }

    /// Write a string to the file
    void write(string data)
    {
        enforce(!isClosed, "Attempt to write to closed file");
        file.write(data);
        file.flush();
    }

    /// Read a line from the file
    string readline()
    {
        enforce(!isClosed, "Attempt to read from closed file");
        import std.algorithm : canFind;
        if (file.eof())
            return null;
        return file.readln();
    }

    /// Read all contents of the file
    string readAll()
    {
        enforce(!isClosed, "Attempt to read from closed file");
        return file.readText();
    }

    /// Check if EOF reached
    bool eof() const
    {
        enforce(!isClosed, "Check eof on closed file");
        return file.eof();
    }
}

/// Wrapper for standard files (stdin, stdout, stderr)
struct LuaStdFile
{
    File* file;

    static LuaStdFile stdinFile() { return LuaStdFile(&stdin); }
    static LuaStdFile stdoutFile() { return LuaStdFile(&stdout); }
    static LuaStdFile stderrFile() { return LuaStdFile(&stderr); }

    void write(string data) { file.write(data); file.flush(); }
    string readline() { return file.readln(); }
    bool eof() const { return file.eof(); }
}

/// Example Lua I/O library functions exposed to Lua VM

/// io.open(filename, mode)
LuaFile io_open(string filename, string mode = "r")
{
    try
    {
        return LuaFile.openFile(filename, mode);
    }
    catch (Exception e)
    {
        // In Lua, return nil plus error message
        // Here, you might want to adapt this to your error handling
        // For demonstration, rethrow
        throw e;
    }
}

/// io.close(file)
void io_close(ref LuaFile file)
{
    file.close();
}

/// io.read(file, count)
string io_read(ref LuaFile file, size_t count = size_t.max)
{
    enforce(!file.isClosed, "Attempt to read from closed file");
    if (count == size_t.max)
        return file.readAll();
    else
        return file.file.read(count);
}

/// io.write(file, data)
void io_write(ref LuaFile file, string data)
{
    file.write(data);
}

/// io.stdin()
LuaStdFile io_stdin()
{
    return LuaStdFile.stdinFile();
}

/// io.stdout()
LuaStdFile io_stdout()
{
    return LuaStdFile.stdoutFile();
}

/// io.stderr()
LuaStdFile io_stderr()
{
    return LuaStdFile.stderrFile();
}
module liolib_bindings;

import core.stdc.stdlib : malloc, free;
import std.string : fromStringz;
import std.conv : to;
import lapi;    // Your Lua API bindings, e.g., lua_State etc.
import liolib;  // Your previously defined I/O library module

// We define a userdata type tag for file handles
enum LUA_FILEHANDLE = 1; // unique id for file userdata metatable

// Push a LuaFile userdata onto the Lua stack
void push_file_userdata(lua_State* L, LuaFile file)
{
    auto ud = cast(LuaFile*) lua_newuserdata(L, LuaFile.sizeof);
    *ud = file;
    // Set metatable for file userdata
    luaL_getmetatable(L, "LuaFile");
    lua_setmetatable(L, -2);
}

// Check and get LuaFile userdata from stack at index
LuaFile* check_file_userdata(lua_State* L, int idx)
{
    return cast(LuaFile*) luaL_checkudata(L, idx, "LuaFile");
}

// Lua wrapper for io.open(filename, mode)
extern(C) int lua_io_open(lua_State* L)
{
    auto filename = luaL_checkstring(L, 1);
    auto mode = luaL_optstring(L, 2, "r");

    try
    {
        auto file = LuaFile.openFile(filename.fromStringz, mode.fromStringz);
        push_file_userdata(L, file);
        return 1;
    }
    catch (Exception e)
    {
        lua_pushnil(L);
        lua_pushstring(L, e.msg.toStringz);
        return 2;
    }
}

// Lua wrapper for file:close()
extern(C) int lua_file_close(lua_State* L)
{
    auto file = check_file_userdata(L, 1);
    if (!file.isClosed)
    {
        file.close();
    }
    lua_pushboolean(L, 1);
    return 1;
}

// Lua wrapper for file:read()
extern(C) int lua_file_read(lua_State* L)
{
    auto file = check_file_userdata(L, 1);
    if (file.isClosed)
    {
        lua_pushnil(L);
        lua_pushstring(L, "attempt to read from closed file".toStringz);
        return 2;
    }
    // Optional argument: count bytes to read
    size_t count = lua_gettop(L) >= 2 ? cast(size_t) lua_tointeger(L, 2) : size_t.max;

    try
    {
        auto result = io_read(*file, count);
        if (result is null)
        {
            lua_pushnil(L);
        }
        else
        {
            lua_pushstring(L, result.toStringz);
        }
        return 1;
    }
    catch (Exception e)
    {
        lua_pushnil(L);
        lua_pushstring(L, e.msg.toStringz);
        return 2;
    }
}

// Lua wrapper for file:write(data)
extern(C) int lua_file_write(lua_State* L)
{
    auto file = check_file_userdata(L, 1);
    if (file.isClosed)
    {
        lua_pushnil(L);
        lua_pushstring(L, "attempt to write to closed file".toStringz);
        return 2;
    }
    auto data = luaL_checkstring(L, 2);
    try
    {
        io_write(*file, data.fromStringz);
        lua_pushboolean(L, 1);
        return 1;
    }
    catch (Exception e)
    {
        lua_pushnil(L);
        lua_pushstring(L, e.msg.toStringz);
        return 2;
    }
}
}
