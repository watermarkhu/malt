% I_BANK_DEMO  driver script for i_account and i_customer
% run this file from the folder that holds the class files

clear all
close all
clc

% --- set up the customer and two accounts ---
alice = i_customer('Alice');
acc1 = i_account('Alice', 100);
acc2 = i_account('Alice');
alice = alice.addAccount(acc1);
alice = alice.addAccount(acc2);

% --- simple deposits ---
acc1.deposit(50);
acc2.deposit(25);
fprintf('after deposits: %s\n', acc1.summary());
fprintf('after deposits: %s\n', acc2.summary());

% --- a withdraw that must fail ---
try
    acc2.withdraw(1000);
catch err
    disp(['caught: ' err.message]);
end

% --- a valid withdraw ---
acc1.withdraw(30);

% --- move money between the two accounts ---
acc1.transfer(acc2, 20);
fprintf('transfer done: %s / %s\n', acc1.summary(), acc2.summary());

% --- aggregate figures ---
fprintf('customer %s has %d accounts\n', alice.Name, alice.numAccounts());
fprintf('total balance: %.2f\n', alice.totalBalance());

% --- loop over accounts the old way ---
for k = 1:numel(alice.Accounts)
    a = alice.Accounts{k};
    fprintf('account %d: %g (%d ops)\n', k, a.Balance, numel(a.History));
end

% --- look one up by name ---
found = alice.findAccount('Alice');
if isempty(found)
    disp('not found');
else
    disp(['found balance ' num2str(found.Balance)]);
end

% --- remove the second account and report ---
alice = alice.removeAccount(2);
alice.report();

% --- error on a bad index ---
try
    alice = alice.removeAccount(7);
catch ex
    fprintf('error id: %s\n', ex.identifier);
end

disp('done');
