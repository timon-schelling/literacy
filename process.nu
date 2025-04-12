
let input = "./gls-3-1-3.txt"
let lang = "de"
let tts_model = "./de_DE-thorsten-high.onnx"
let input_name = $input | path basename | split row "." | first
let processing_dir = $"($input_name).processing"
let result_dir = $"($input_name).result"
let tts_dir = $"($processing_dir)/tts"

def preprocess []: string -> list<string> {
    $in
        | str replace ". " ".\n"
        | str replace "!" "!\n"
        | str replace "?" "?\n"
        | str replace ": " ":\n"
        | str replace "  " " "
        | lines
        | each { $in | str trim }
        | filter { $in != "" }
}

def romanize []: list<string> -> list<string> {
    $in | uroman -l deu | lines | each { $in | str replace "-" "" }
}

def tts [tts_model: path, $tts_dir: path = ./tts_output]: list<string> -> list<path> {
    mkdir $tts_dir
    $in | str join "\n" | piper --model $tts_model --output_dir $tts_dir
    ls $tts_dir | sort-by modified | get name
}

def compress [$compress_dir: path = ./compress_output]: list<path> -> list<path> {
    mkdir $compress_dir
    $in | each { |it|
        let name = $it | path basename | split row "." | first
        ffmpeg -i $it -codec:a libmp3lame -b:a 128k -ar 44100 -ac 1 -qscale:a 2 $"($compress_dir)/($name).mp3"
        $"($compress_dir)/($name).mp3"
    }
}

def align [audio: path, $align_dir: path = ./align_tmp]: string -> record {
    let hash = $in | hash sha256
    let txt_file = $"($align_dir)/($hash).txt"
    let wav_file = $"($align_dir)/($hash).wav"
    let json_file = $"($align_dir)/($hash).json"
    mkdir $align_dir
    $in | save -f $txt_file
    cp $audio $wav_file
    ctc-forced-aligner --language deu --audio_path $audio --text_path $txt_file
    open $json_file | get segments | select text start end | rename content
}

def main [input: path, lang: string, tts_model: path, result_dir: path, result_name: string] {
    let input_name = $input | path basename | split row "." | first
    let processing_dir = $"($input_name)_processing"
    let preprocessed = open $input | preprocess
    mkdir $processing_dir
    $preprocessed | save -f $"($processing_dir)/lines.txt"
    let tts_files = $preprocessed | tts $tts_model $"($processing_dir)/tts"
    let compressed_files = $tts_files | compress $"($processing_dir)/compressed"
    mkdir $result_dir
    let segments = $preprocessed | zip $compressed_files | each { |it|
        let text = $it.0
        let audio = $it.1

        let audio_name = $audio | path basename
        cp $audio $"($result_dir)/($audio_name)"

        let words = $text | split row " "

        let duration = ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 $audio | into float

        { audio: { "ref": $audio_name }, "words": $words, "duration": $duration }
    }
    { "segments": $segments } | save -f $"($result_dir)/($result_name).json"
}

# mkdir $"($processing_dir)/tts"

# open $"($processing_dir)/lines.txt" | piper --model $tts_model --output_dir $"($processing_dir)/tts"
# ls $"($processing_dir)/tts" | get name | enumerate | par-each { |it|
#     mv $"($it.item)" $"($processing_dir)/tts/($it.index).wav"
# }

# open $"($processing_dir)/lines.txt" | uroman -l deu | lines | each { $in | str replace "-" "" } | save -f $"($processing_dir)/lines_romanized.txt"

# let lines = open $"($processing_dir)/lines.txt" | lines
# let lines_romanized = open $"($processing_dir)/lines_romanized.txt" | lines

# mkdir $"($processing_dir)/align"
# ls $"($processing_dir)/tts" | get name | par-each {
#     let name = $in | path basename | split row "." | first
#     cp $"($in)" $"($processing_dir)/align/($name).wav"
#     $lines_romanized | get ($name | into int) | save -f $"($processing_dir)/align/($name).txt"
#     ctc-forced-aligner --language deu --audio_path $"($processing_dir)/align/($name).wav" --text_path $"($processing_dir)/align/($name).txt"
#     let res = open $"($processing_dir)/align/($name).json" | get segments | select text start end | rename romanized_content
#     let words = $lines | get ($name | into int) | split row " "
#     let segments = $res | enumerate | each { |it| $it.item | insert content ($words | get $it.index) } | select content start end romanized_content
#     let segments = $segments | insert distance { |it| $it.content | str distance $it.romanized_content }
#     $segments | save -f $"($processing_dir)/align/($name).json"
#     rm $"($processing_dir)/align/($name).txt"
#     rm $"($processing_dir)/align/($name).wav"
# }

# mkdir $"($processing_dir)/compressed"
# ls $"($processing_dir)/tts" | get name | par-each {
#     let name = $in | path basename | split row "." | first
#     ffmpeg -i $in -codec:a libmp3lame -b:a 128k -ar 44100 -ac 1 -qscale:a 2 $"($processing_dir)/compressed/($name).mp3"
# }

# mkdir $"($processing_dir)/segments"
# ls $"($processing_dir)/align" | get name | par-each {
#     let name = $in | path basename | split row "." | first
#     let words = open $in | select content start end
#     { audio: { "ref": $"($name).mp3" }, "words": $words } | save -f $"($processing_dir)/segments/($name).json"
# }

# try { rm -r $result_dir }
# mkdir $result_dir
# ls $"($processing_dir)/compressed" | get name | par-each {
#     cp $in $result_dir
# }
# { "segments": (ls $"($processing_dir)/segments" | sort-by name -n | get name | each { open $in }) } | save -f $"($result_dir)/text.json"


# nu ../process.nu gls-3-1-3.txt de de_DE-thorsten-high.onnx ./result gls-3-1-3
