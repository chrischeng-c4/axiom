import json
import typer

from task_cli.application.create_task import create_task

app = typer.Typer(add_completion=False)


@app.callback()
def task_cli() -> None:
    """Public task command boundary."""


@app.command()
def create(title: str, actor: str = "system") -> None:
    try:
        task = create_task(title, actor)
    except ValueError as error:
        typer.echo(json.dumps({"error": str(error)}, sort_keys=True))
        raise typer.Exit(code=2)
    typer.echo(json.dumps({"actor": task.actor, "title": task.title}, sort_keys=True))


if __name__ == "__main__":
    app()
