# literacy

## notes

### manual generation

#### audio

#### timestamps

```nu
whisperx --compute_type int8 --print_progress True --max_line_width 50 --segment_resolution chunk --max_line_count 1 --language de assets/audio.wav

let original = open assets/text.txt | split row " " | split row "\n" | str trim | filter { $in != "" }
let processed = open audio.json | get word_segments | select word start end | rename content | enumerate | each { |e| $e.item | insert original ($original | get $e.index) | insert distance { |f| $e.item.content | str distance ($f.original) } }
let words = $processed | select original start end | rename content
{ "words": $words, audio: { "ref": "audio.wav" } } | save -f segment.json
```
