import zipfile, sys
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
def para(text, code=False):
    style = '<w:pPr><w:pStyle w:val="matlab-Code"/></w:pPr>' if code else '<w:pPr><w:pStyle w:val="Normal"/></w:pPr>'
    return f'<w:p>{style}<w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>'
body = "".join([
    para("Sinusoidal signals"),
    para("A short intro, with an accent: caf\u00e9."),
    para("x = linspace(0, 8*pi);", True),
    para("", False),
    para("y = sin(x);", True),
    para("plot(x, y)", True),
    para("Summary text."),
    para("Last code line;", True),
])
xml = f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<w:document xmlns:w="{W}"><w:body>{body}</w:body></w:document>'
with zipfile.ZipFile("mlx/Shot.mlx", "w") as zf:
    zf.writestr("matlab/document.xml", xml)
    zf.writestr("metadata/mwcoreProperties.xml", "<x/>")
