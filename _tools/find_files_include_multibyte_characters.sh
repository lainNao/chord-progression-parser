# find files including multibyte characters inside src directory
# Usage: ./_tools/find_files_include_multibyte_characters.sh

find src -type f | LANG=C xargs grep -n -v '^[[:cntrl:][:print:]]*$' --binary-files=without-match