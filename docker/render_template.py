import os, argparse, json

import jinja2

args_parser = argparse.ArgumentParser()
args_parser.add_argument('template_file', help='Jinja2 template file to render.')
args_parser.add_argument('render_vars', help='JSON-encoded data to pass to the templating engine.')
cli_args = args_parser.parse_args()

render_vars = json.loads(cli_args.render_vars)
environment = jinja2.Environment(
  loader=jinja2.FileSystemLoader(os.getcwd()),
  trim_blocks=True,
)
print(environment.get_template(cli_args.template_file).render(render_vars))
