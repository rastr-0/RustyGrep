# RustyGrep
## Description
RustyGrep is a command-line tool written in Rust that implements basic search functionality.
The final main goal is RustyGrep to be more or less similar in its functionality to Unix "grep".
Found words or the parts of words are colored in red.
## Available LOCAL_VARIABLES
All the local variables can be combined
```
IGNORE_CASE=1 -- search works in an insensitive case format
FULL_WORDS=1 -- search returns matches with full words
FULL_LINES=1 -- search returns matches with full lines
INVERT_MATCH=1 -- search returns lines where match was not found
MAX_OUTPUT=N -- search returns first N matched lines 
```
## Usage
To usage the utility navigate to the project directory and run the following command:
```your_local_variables(optional) cargo run <search_item> <file_path>```
## Examples
poem.txt file:
```
Two roads diverged in a yellow wood,
And sorry I could not travel both
And be one traveler, long I stood
And looked down one as far as I could
To where it bent in the undergrowth.

Then took the other, as just as fair,
And having perhaps the better claim,
Because it was grassy and wanted wear;
Though as for that the passing there
Had worn them really about the same.

And both that morning equally lay
In leaves no step had trodden black.
Oh, I kept the first for another day!
Yet knowing how way leads on to way,
I doubted if I should ever come back.
```
* FULL_WORDS=1 cargo run the poem.txt
```
searching for "the" in the file: poem.txt
line: 5 | To where it bent in the undergrowth.
line: 7 | Then took the other, as just as fair,
line: 8 | And having perhaps the better claim,
line: 10 | Though as for that the passing there
line: 11 | Had worn them really about the same.
line: 15 | Oh, I kept the first for another day!
```
* FULL_WORDS=1 IGNORE_CASE=1 cargo run and poem.txt
```
searching for "and" in the file: poem.txt
line: 2 | And sorry I could not travel both
line: 3 | And be one traveler, long I stood
line: 4 | And looked down one as far as I could
line: 8 | And having perhaps the better claim,
line: 9 | Because it was grassy and wanted wear;
line: 13 | And both that morning equally lay
```
## TODO List
- [X] Basic search
- [X] Register sensivity
- [X] Only full words sensivity
- [X] Full test coverage 
- [X] Max count output lines
- [X] Invert match
- [X] Only full lines sensitivity
- [ ] Basic regular expressions support
- [ ] Async search of matches in more than 1 file  

