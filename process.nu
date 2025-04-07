let input = "./text.txt"
let lang = "de"
let tts_model = "./de_DE-thorsten-high.onnx"
let input_name = $input | path basename | split row "." | first
let processing_dir = $"($input_name).processing"
let result_dir = $"($input_name).result"

try { rm -r $processing_dir }

mkdir $processing_dir

open $input | split row "\n" | filter { ($in | str trim) != "" } | save -f $"($processing_dir)/lines.txt"

mkdir $"($processing_dir)/tts"

open $"($processing_dir)/lines.txt" | piper --model $tts_model --output_dir $"($processing_dir)/tts"
ls $"($processing_dir)/tts" | get name | enumerate | par-each { |it|
    mv $"($it.item)" $"($processing_dir)/tts/($it.index).wav"
}

open $"($processing_dir)/lines.txt" | uroman -l deu | lines | each { $in | str replace "-" "" } | save -f $"($processing_dir)/lines_romanized.txt"

let lines = open $"($processing_dir)/lines.txt" | lines
let lines_romanized = open $"($processing_dir)/lines_romanized.txt" | lines

mkdir $"($processing_dir)/align"
ls $"($processing_dir)/tts" | get name | par-each {
    let name = $in | path basename | split row "." | first
    cp $"($in)" $"($processing_dir)/align/($name).wav"
    $lines_romanized | get ($name | into int) | save -f $"($processing_dir)/align/($name).txt"
    ctc-forced-aligner --language deu --audio_path $"($processing_dir)/align/($name).wav" --text_path $"($processing_dir)/align/($name).txt"
    let res = open $"($processing_dir)/align/($name).json" | get segments | select text start end | rename romanized_content
    let words = $lines | get ($name | into int) | split row " "
    let segments = $res | enumerate | each { |it| $it.item | insert content ($words | get $it.index) } | select content start end romanized_content
    let segments = $segments | insert distance { |it| $it.content | str distance $it.romanized_content }
    $segments | save -f $"($processing_dir)/align/($name).json"
    rm $"($processing_dir)/align/($name).txt"
    rm $"($processing_dir)/align/($name).wav"
}

mkdir $"($processing_dir)/compressed"
ls $"($processing_dir)/tts" | get name | par-each {
    let name = $in | path basename | split row "." | first
    ffmpeg -i $in -codec:a libmp3lame -b:a 128k -ar 44100 -ac 1 -qscale:a 2 $"($processing_dir)/compressed/($name).mp3"
}

mkdir $"($processing_dir)/segments"
ls $"($processing_dir)/align" | get name | par-each {
    let name = $in | path basename | split row "." | first
    let words = open $in | select content start end
    { audio: { "ref": $"($name).wav" }, "words": $words } | save -f $"($processing_dir)/segments/($name).json"
}

try { rm -r $result_dir }
mkdir $result_dir
ls $"($processing_dir)/compressed" | get name | par-each {
    cp $in $result_dir
}
ls $"($processing_dir)/segments" | get name | par-each {
    cp $in $result_dir
}
