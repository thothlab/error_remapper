package com.thothlab.remapper;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/**
 * Java wrapper for the error-remapper Rust shared library.
 *
 * Usage:
 * <pre>
 *   ErrorRemapper remapper = new ErrorRemapper("/path/to/config");
 *   String result = remapper.remap("{\"statusCode\": \"3011\", \"errorText\": \"Не пройден фрод\"}");
 *   System.out.println(result);
 * </pre>
 *
 * The config directory must contain:
 *   - settings.toml
 *   - errors.yaml (or the path specified in settings.toml)
 */
public class ErrorRemapper {

    /**
     * JNA interface to the native library.
     */
    private interface NativeLib extends Library {
        Pointer error_remapper_remap(String inputJson, String configDir);
        void error_remapper_free(Pointer s);
    }

    private final NativeLib lib;
    private final String configDir;

    /**
     * Create a new ErrorRemapper instance.
     *
     * @param configDir path to directory containing settings.toml and errors.yaml
     */
    public ErrorRemapper(String configDir) {
        this.configDir = configDir;
        this.lib = Native.load("error_remapper", NativeLib.class);
    }

    /**
     * Create a new ErrorRemapper with explicit library path.
     *
     * @param libraryPath full path to liberror_remapper.dylib / .so
     * @param configDir   path to directory containing settings.toml and errors.yaml
     */
    public ErrorRemapper(String libraryPath, String configDir) {
        this.configDir = configDir;
        this.lib = Native.load(libraryPath, NativeLib.class);
    }

    /**
     * Remap an error JSON string.
     *
     * @param inputJson JSON string with the error (any format, fields configured in settings.toml)
     * @return JSON string with the remapped result (format configured in settings.toml)
     */
    public String remap(String inputJson) {
        Pointer ptr = lib.error_remapper_remap(inputJson, configDir);
        try {
            return ptr.getString(0, "UTF-8");
        } finally {
            lib.error_remapper_free(ptr);
        }
    }
}
