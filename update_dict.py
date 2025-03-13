#!/usr/bin/env python3
"""Update OpenCC directories (public/{s2t,t2s}.txt)"""

from pathlib import Path
import requests

DICT_URL = "https://raw.githubusercontent.com/BYVoid/OpenCC/refs/heads/master/data/dictionary/"
OUTPUT_DIR = "public/"

DICTS = {
    "s2t.txt": ["STCharacters.txt", "STPhrases.txt"],
    "t2s.txt": ["TSCharacters.txt", "TSPhrases.txt"],
}


def main():
    for output_name, dict_files in DICTS.items():
        output = Path(OUTPUT_DIR) / Path(output_name)
        with output.open('w', encoding='utf-8') as out:
            for dict_file in dict_files:
                url = f"{DICT_URL}{dict_file}"
                print('Fetch', url)
                with requests.get(url, stream=True) as resp:
                    for line in resp.iter_lines():
                        preimage, images = line.decode('utf-8').strip().split('\t')
                        out.write(f"{preimage}\t{images.split()[0]}\n")
        print('Dict updated', output)


if __name__ == "__main__":
    main()
