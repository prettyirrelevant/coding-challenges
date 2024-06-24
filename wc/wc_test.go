package main

import (
	"bufio"
	"io"
	"strings"
	"testing"
	"unicode"
)

func TestCountAll(t *testing.T) {
	tests := []struct {
		name       string
		input      string
		countLines bool
		countWords bool
		countBytes bool
		countChars bool
		expected   struct {
			lines int
			words int
			bytes int
			chars int
		}
	}{
		{
			name:       "Empty string",
			input:      "",
			countLines: true,
			countWords: true,
			countBytes: true,
			countChars: true,
			expected:   struct{ lines, words, bytes, chars int }{0, 0, 0, 0},
		},
		{
			name:       "Single line",
			input:      "Hello World",
			countLines: true,
			countWords: true,
			countBytes: true,
			countChars: true,
			expected:   struct{ lines, words, bytes, chars int }{0, 2, 11, 11},
		},
		{
			name:       "Multiple lines",
			input:      "Hello World\nThis is a test\n",
			countLines: true,
			countWords: true,
			countBytes: true,
			countChars: true,
			expected:   struct{ lines, words, bytes, chars int }{2, 6, 27, 27},
		},
		{
			name:       "Count only lines",
			input:      "Hello\nWorld\n",
			countLines: true,
			countWords: false,
			countBytes: false,
			countChars: false,
			expected:   struct{ lines, words, bytes, chars int }{2, 0, 0, 0},
		},
		{
			name:       "Count only words",
			input:      "Hello World",
			countLines: false,
			countWords: true,
			countBytes: false,
			countChars: false,
			expected:   struct{ lines, words, bytes, chars int }{0, 2, 0, 0},
		},
		{
			name:       "Count only bytes",
			input:      "Hello",
			countLines: false,
			countWords: false,
			countBytes: true,
			countChars: false,
			expected:   struct{ lines, words, bytes, chars int }{0, 0, 5, 0},
		},
		{
			name:       "Count only characters",
			input:      "Hello",
			countLines: false,
			countWords: false,
			countBytes: false,
			countChars: true,
			expected:   struct{ lines, words, bytes, chars int }{0, 0, 0, 5},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			lines, words, bytes, chars, err := CountAllFromString(tt.input, tt.countLines, tt.countWords, tt.countBytes, tt.countChars)
			if err != nil {
				t.Fatalf("CountAllFromString() error = %v", err)
			}
			if lines != tt.expected.lines {
				t.Errorf("CountAllFromString() lines = %v, want %v", lines, tt.expected.lines)
			}
			if words != tt.expected.words {
				t.Errorf("CountAllFromString() words = %v, want %v", words, tt.expected.words)
			}
			if bytes != tt.expected.bytes {
				t.Errorf("CountAllFromString() bytes = %v, want %v", bytes, tt.expected.bytes)
			}
			if chars != tt.expected.chars {
				t.Errorf("CountAllFromString() chars = %v, want %v", chars, tt.expected.chars)
			}
		})
	}
}

func CountAllFromString(input string, countLines, countWords, countBytes, countChars bool) (lines, words, bytes, chars int, err error) {
	reader := strings.NewReader(input)
	bufReader := bufio.NewReader(reader)
	inWord := false

	for {
		r, size, err := bufReader.ReadRune()
		if err != nil {
			if err == io.EOF {
				break
			}
			return 0, 0, 0, 0, err
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

func TestCountAllFromFile(t *testing.T) {
	filePath := "test.txt"
	lines, words, bytes, chars, err := CountAll(filePath, true, true, true, true)
	if err != nil {
		t.Fatalf("CountAll() error = %v", err)
	}

	expected := struct{ lines, words, bytes, chars int }{7143, 58164, 342143, 339245}
	if lines != expected.lines {
		t.Errorf("CountAll() lines = %v, want %v", lines, expected.lines)
	}
	if words != expected.words {
		t.Errorf("CountAll() words = %v, want %v", words, expected.words)
	}
	if bytes != expected.bytes {
		t.Errorf("CountAll() bytes = %v, want %v", bytes, expected.bytes)
	}
	if chars != expected.chars {
		t.Errorf("CountAll() chars = %v, want %v", chars, expected.chars)
	}
}
