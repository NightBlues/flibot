import sqlite3
import hashlib
import os


con = sqlite3.connect("db.sqlite")
cur = con.cursor()
cur.execute("ALTER TABLE books ADD COLUMN fb2_sha1 VARCHAR(255)")
con.commit()
res = cur.execute("SELECT id, title, fb2_sha1, fb2 from books")
books = res.fetchall()

def sha1sum(data):
    sha = hashlib.sha1()
    sha.update(data)
    return sha.hexdigest()


for book in books:
    id, title, fb2_sha1, fb2 = book
    if fb2 is None or fb2_sha1 is not None:
        continue
    print(id, title)
    new_sha1 = sha1sum(fb2)
    print(new_sha1)
    basedir = "files/{}".format(new_sha1[:2])
    os.makedirs(basedir, exist_ok=True)
    filename = basedir + "/" + new_sha1
    print(filename)
    with open(filename, "wb") as fp:
        fp.write(fb2)
    cur.execute(
        "UPDATE books SET fb2_sha1=? WHERE id=?",
        (new_sha1, id))
    con.commit()
    # check
    with open(filename, "rb") as fp:
        data = fp.read()
    ck_sha1 = sha1sum(data)
    print("check sha1: ", ck_sha1)
    if ck_sha1 != new_sha1:
        break
    
