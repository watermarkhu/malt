function balance = bank_account(ops)
%BANK_ACCOUNT Applies a list of deposit and withdrawal operations.
%   ops is a cell array of {type, amount} rows. Overdrafts are refused.

balance = 0;
history = cell(numel(ops), 2);
refused = 0;
for k = 1:numel(ops)
    kind = ops{k}{1};
    amount = ops{k}{2};
    switch kind
        case 'deposit'
            balance = balance + amount;
            status = 'ok';
        case 'withdraw'
            if amount > balance
                status = 'refused';
                refused = refused + 1;
            else
                balance = balance - amount;
                status = 'ok';
            end
        otherwise
            error('bank:kind', 'unknown operation "%s"', kind);
    end
    history(k, :) = {kind, status};
end

okRows = history(strcmp(history(:, 2), 'ok'), 1);
fprintf('%d operations, %d refused\n', size(history, 1), refused);
fprintf('deposits: %d\n', sum(strcmp(okRows, 'deposit')));

interest = 0;
if balance > 1000
    interest = balance * 0.01;
    balance = balance + interest;
end
rate = 0.02;
fees = [];
for m = 1:12
    fees(m) = balance * rate / 12;
end
totalFees = sum(fees)
if totalFees > 0
    balance = balance - totalFees;
end
end
