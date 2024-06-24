package main

import (
	"bufio"
	"flag"
	"fmt"
	"io"
	"os"
	"unicode"
)

func main() {
	linePtr := flag.Bool("l", false, "Count lines")
	wordPtr := flag.Bool("w", false, "Count words")
	bytesPtr := flag.Bool("c", false, "Count bytes")
	charPtr := flag.Bool("m", false, "Count characters")

	flag.Parse()

	filePath := flag.Arg(0)
	noFlagSet := !(*linePtr || *wordPtr || *bytesPtr || *charPtr)
	countLines := *linePtr || noFlagSet
	countWords := *wordPtr || noFlagSet
	countBytes := *bytesPtr || noFlagSet
	countChars := *charPtr && !noFlagSet
	lines, words, bytes, chars, err := CountAll(filePath, countLines, countWords, countBytes, countChars)
	if err != nil {
		fmt.Printf("wcc: error: %v\n", err)
		os.Exit(1)
	}

	FormatResult(lines, words, bytes, chars, filePath)
}

func FormatResult(lines, words, bytes, chars int, filePath string) {
	result := ""
	if lines > 0 {
		result += fmt.Sprintf("%8d", lines)
	}
	if words > 0 {
		result += fmt.Sprintf("%8d", words)
	}
	if chars > 0 {
		result += fmt.Sprintf("%8d", chars)
	} else if bytes > 0 {
		result += fmt.Sprintf("%8d", bytes)
	}
	if filePath != "" {
		result += fmt.Sprintf(" %s", filePath)
	}

	fmt.Println(result)
}

func CountAll(filePath string, countLines, countWords, countBytes, countChars bool) (lines, words, bytes, chars int, err error) {
	var reader io.Reader
	if filePath == "" {
		reader = os.Stdin
	} else {
		file, err := os.Open(filePath)
		if err != nil {
			return 0, 0, 0, 0, fmt.Errorf("wcc: failed to open file: %v", err)
		}
		defer file.Close()
		reader = file
	}

	bufReader := bufio.NewReader(reader)
	inWord := false

	for {
		r, size, err := bufReader.ReadRune()
		if err != nil {
			if err == io.EOF {
				break
			}
			return 0, 0, 0, 0, fmt.Errorf("wcc: failed to read: %v", err)
		}

		if countBytes {
			bytes += size
		}

		if countChars {
			chars++
		}

		if countLines && r == '\n' {
			lines++
		}

		if countWords {
			if unicode.IsSpace(r) {
				inWord = false
			} else if !inWord {
				words++
				inWord = true
			}
		}
	}

	return lines, words, bytes, chars, nil
}
