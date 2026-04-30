# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'ProvChainOrg'
copyright = '2025, ProvChainOrg Team'
author = 'ProvChainOrg Team'
release = '0.1.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',           # Auto-generate from docstrings
    'sphinx.ext.viewcode',          # Source code links
    'sphinx.ext.napoleon',          # Google/NumPy docstring support
    'sphinxcontrib.plantuml',       # PlantUML integration
    'sphinx_rtd_theme',             # ReadTheDocs theme
    'sphinx.ext.intersphinx',       # Cross-project references
    'sphinx_copybutton',            # Copy code button
    'myst_parser',                  # Markdown support
    'sphinx.ext.todo',              # TODO notes
    'sphinx.ext.ifconfig',          # Conditional content
]

templates_path = ['_templates']
exclude_patterns = [
    '_build', 
    'Thumbs.db', 
    '.DS_Store',
    '*.md',  # Exclude all markdown files for now
    'test_build.py',
]

# -- Options for HTML output ------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']

# -- PlantUML configuration -------------------------------------------------
plantuml = 'java -jar plantuml.jar'
plantuml_output_format = 'svg'

# -- Theme options -----------------------------------------------------------
html_theme_options = {
    'canonical_url': '',
    'analytics_id': '',
    'logo_only': False,
    'display_version': True,
    'prev_next_buttons_location': 'bottom',
    'style_external_links': False,
    'vcs_pageview_mode': '',
    'style_nav_header_background': '#2980B9',
    # Toc options
    'collapse_navigation': True,
    'sticky_navigation': True,
    'navigation_depth': 4,
    'includehidden': True,
    'titles_only': False
}

# -- Custom CSS -------------------------------------------------------------
html_css_files = [
    'custom.css',
    'interactive.css',
]

# -- Custom JS --------------------------------------------------------------
html_js_files = [
    'interactive.js',
]

# -- LaTeX output options ---------------------------------------------------
latex_elements = {
    'papersize': 'letterpaper',
    'pointsize': '10pt',
    'preamble': r'''
\usepackage{charter}
\usepackage[defaultsans]{lato}
\usepackage{inconsolata}
''',
}

latex_documents = [
    ('index', 'ProvChainOrg.tex', 'ProvChainOrg Documentation',
     'ProvChainOrg Team', 'manual'),
]

# -- EPUB output options ----------------------------------------------------
epub_title = project
epub_exclude_files = ['search.html']

# -- Intersphinx mapping ----------------------------------------------------
intersphinx_mapping = {
    'python': ('https://docs.python.org/3/', None),
}

# -- MyST configuration -----------------------------------------------------
myst_enable_extensions = [
    "deflist",
    "tasklist",
    "colon_fence",
]

# -- Todo configuration -----------------------------------------------------
todo_include_todos = True

# -- Source file suffixes ---------------------------------------------------
source_suffix = {
    '.rst': None,
    '.md': 'myst_parser',
}

# -- Master document --------------------------------------------------------
master_doc = 'index'

# -- Version info ------------------------------------------------------------
# The short X.Y version
version = '0.1'
# The full version, including alpha/beta/rc tags
release = '0.1.0'

# -- Custom configuration ---------------------------------------------------
# Add any custom configuration here
numfig = True
numfig_format = {
    'figure': 'Figure %s',
    'table': 'Table %s',
    'code-block': 'Listing %s',
    'section': 'Section %s',
}
