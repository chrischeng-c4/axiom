# RUN: parse

class Animal:
    name: str = ""
    def speak(self) -> str:
        return "..."

class Dog(Animal):
    def speak(self) -> str:
        return "woof"
