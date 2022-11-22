-- "date": date,
-- "origin": sql_o,
-- "origin_vol": sql_o_vol,
-- "destination": sql_d,
-- "destination_vol": sql_d_vol,
-- "amount": amount,
-- "denom": denom.to_uppercase().as_str(),
-- "price": price,
-- "address": secret_address,
LET $date = "2022_11_21";
LET $origin = chain:axelar;
LET $origin_vol = axelar:scrt;
LET $destination = chain:osmosis;
LET $destination_vol = osmosis:scrt;
LET $amount = 1000;
LET $denom = "SCRT";
LET $price = 50.0;
LET $address = "secret1qew9dn4058fdfh32osddjfewj";

BEGIN TRANSACTION;
    LET $uuid = rand::uuid();
    LET $t = (CREATE transaction SET datetime = $date, origin = $origin, destination = $destination, amount = $amount, denom = $denom, uuid = $uuid, price = $price);
    UPDATE $origin SET outgoing_transactions +=1, transactions += $t.id;
    UPDATE $destination SET incoming_transactions +=1, transactions = array::union(transactions, [$t.id]);
    UPDATE $origin_vol SET volume += { date: $date, incoming: 0, outgoing: 0 } WHERE volume[$].date != $date;
    UPDATE $origin_vol SET volume[$].outgoing += $amount, total_outgoing += $amount WHERE volume[$].date = $date;
    UPDATE $destination_vol SET volume += { date: $date, incoming: 0, outgoing: 0 } WHERE volume[$].date != $date;
    UPDATE $destination_vol SET volume[$].incoming += $amount, total_incoming += $amount WHERE volume[$].date = $date;
    INSERT INTO wallet (id, address) VALUES ($address, $address);
COMMIT TRANSACTION;