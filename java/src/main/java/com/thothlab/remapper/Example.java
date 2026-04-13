package com.thothlab.remapper;

/**
 * Usage example for ErrorRemapper.
 *
 * Run:
 *   java -Djna.library.path=/path/to/target/release -cp .:jna.jar com.thothlab.remapper.Example
 */
public class Example {

    public static void main(String[] args) {
        // Path to directory with settings.toml and errors.yaml
        String configDir = args.length > 0 ? args[0] : "config";

        ErrorRemapper remapper = new ErrorRemapper(configDir);

        // Example 1: statusCode format
        String input1 = "{\"statusCode\": \"3011\", \"errorText\": \"Не пройден фрод\", \"ErrorDescription\": \"Процесс не был пройден через антифрод\"}";
        System.out.println("Input:  " + input1);
        System.out.println("Output: " + remapper.remap(input1));
        System.out.println();

        // Example 2: nested error format
        String input2 = "{\"error\": {\"code\": \"2001\", \"title\": \"Got unexpected symbol: @ in input\"}}";
        System.out.println("Input:  " + input2);
        System.out.println("Output: " + remapper.remap(input2));
        System.out.println();

        // Example 3: no match
        String input3 = "{\"code\": \"9999\", \"message\": \"Unknown error\"}";
        System.out.println("Input:  " + input3);
        System.out.println("Output: " + remapper.remap(input3));
    }
}
