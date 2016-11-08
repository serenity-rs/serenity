import glob

for filename in glob.glob("target/doc/serenity/**/*.html"):
    print('Parsing {}'.format(filename))
    with open(filename) as f:
        content = f.read()

    new_content = content.replace('<nav class="sidebar">\n', '<nav class="sidebar"><img src="https://docs.austinhellyer.me/serenity.rs/docs_header.png">\n', 1)

    if new_content != content:
        with open(filename, 'w') as f:
            f.write(new_content)
