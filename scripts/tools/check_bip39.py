import re

with open('src/wallet/src/crypto.rs', 'r') as f:
    content = f.read()

match = re.search(r'const BIP39_ENGLISH: \[&str; 2048\] = \[(.*?)\];', content, re.DOTALL)
if match:
    text = match.group(1)
    words = re.findall(r'"(\w+)"', text)
    print(f'Total words in array: {len(words)}')
    unique = sorted(set(words))
    print(f'Unique words: {len(unique)}')
    
    from collections import Counter
    counts = Counter(words)
    dupes = [w for w, c in counts.items() if c > 1]
    if dupes:
        print(f'Duplicates: {dupes[:20]}')
    else:
        print('No duplicates found')
    
    # The official BIP39 wordlist has exactly 2048 words
    if len(words) != 2048:
        print(f'Wordlist is incomplete: need {2048 - len(words)} more words')
    if len(unique) != 2048:
        print(f'Unique count mismatch: {len(unique)} unique words')