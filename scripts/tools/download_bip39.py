import urllib.request
import re

url = 'https://raw.githubusercontent.com/bitcoin/bips/master/bip-0039/english.txt'
with urllib.request.urlopen(url) as response:
    official_words = response.read().decode('utf-8').strip().split('\n')
    official_words = [w.strip() for w in official_words if w.strip()]
    print('Official BIP39 wordlist has exactly', len(official_words), 'words')
    assert len(official_words) == 2048, "Should be 2048"
    
    # Read current Rust file
    with open('src/wallet/src/crypto.rs', 'r') as f:
        content = f.read()
    
    match = re.search(r'const BIP39_ENGLISH: \[&str; 2048\] = \[(.*?)\];', content, re.DOTALL)
    if match:
        text = match.group(1)
        current = re.findall(r'"(\w+)"', text)
        current_set = set(current)
        missing = [w for w in official_words if w not in current_set]
        extra = [w for w in current if w not in official_words]
        print('Missing from our list:', missing)
        print('Extra in our list:', extra)
        
        # Generate the proper wordlist as Rust array
        print('\nGenerating proper BIP39_ENGLISH array...')
        rust_array = 'const BIP39_ENGLISH: [&str; 2048] = [\n'
        for i, word in enumerate(official_words):
            if i % 10 == 0:
                rust_array += '    '
            rust_array += '"' + word + '", '
            if i % 10 == 9:
                rust_array += '\n'
        # Handle remaining
        if len(official_words) % 10 != 0:
            rust_array += '\n'
        rust_array += '];\n'
        print(rust_array[:200] + '...')
        
        # Write to a separate file
        with open('scripts/tools/bip39_wordlist_output.txt', 'w') as of:
            of.write(rust_array)
        print('Wrote to scripts/tools/bip39_wordlist_output.txt')