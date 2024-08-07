class BankAccount:
    def __init__(self, owner, account_number, initial_balance):
        self.owner = owner
        self.account_number = account_number
        self.total = initial_balance

    def add(self, v):
        self.total += v

    def withdraw(self, amount):
        if amount > self.total:
            raise ValueError("Insufficient funds.")
        self.total -= amount
        print(f"Withdrew {amount} from {self.owner}'s account.")

    def check_balance(self):
        print(f"{self.owner}'s account balance: ${self.total:.2f}")
        pass
