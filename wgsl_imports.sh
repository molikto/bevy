rg define_import_path -g '*.wgsl' --sort path | sd '^([^:]*):#define_import_path (.*)' ' "$2": "https://raw.githubusercontent.com/bevyengine/bevy/refs/tags/v0.17.1/$1",' >> output.txt
