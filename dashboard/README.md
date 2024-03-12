PPM-SERVER

* Never re-write the file, use a new timestamped file
* Clean the file history
* Implement the deletion (active y/n)

PPM (client)

* Put the ppm-poc (client) source into the group folder with ppm-server
* Rename the ppm-poc project to ppm
* Login box
* Change / New box
* Upper status bar (login, file timestamp)

Implement the central server communication with a matching code

- User :
- Clé :
- Mot de passe :
  (une seule fois)

Every time we have a new device to peer, generate a matching code.

The matching will stay valid until
we receive incorrect sync data during the sync


#### The User install ppm and launch it


#### @CLOUD_LOGIN - PPM connects to the cloud account to validate the existence of the remote account and logs in

* The account.id + the device.id + DK are sent to server for login request
  * The Cloud checks the inputs and the DK against the HDK.
  * 1. The ACK + A transaction number (TN) is returned. The TN is time limited and single usage.
  * 2. A NACK is returned
  * 3. A PENDING_PAIRING is returned to PPM because the machine id is unknown.

  
#### The User creates a "user account"

* LOCAL The email address and a password is required (OP)
* CLOUD The email address will be validated by return code by email, an account number is generated
* LOCAL A local machine id is generated (uuid), and locally stored in the machine.id file
  * CLOUD A Derived Key (DK) is LOCALLY generated from the UK (UK = HASH(OP)) 
        and sent to the cloud for storage of the HDK (HDK = HASH(DK))
* LOCAL The email + account_id + machine_id will be stored in the account.json file. 
* LOCAL A local encrypted file is created (account_id_<timestamp>), ready to store the entries

#### The User locally logs in

* LOCAL User enters Email and Password
* LOCAL ppm reads the account.json file to determine the account id and machine id (-> memory properties)
* LOCAL ppm creates the user key (UK) by UK = HASH(OP)
* LOCAL The secret file is trying to be decrypted with the UK if fails, login fails.
* LOCAL The UK is stored in the memory properties at the account id entry.


#### NEW_ENTRY - The User creates a new entry

* LOCAL A distributed uuid is generated from machine-id + "-" + Base36( timestamp, local-sequence)
* LOCAL The local file is updated with the new entry
* CLOUD @CLOUD_LOGIN + The new segment + (account.id + machine.id + TN) is sent to the cloud


#### RECONNECT_CLOUD - The User connects to the cloud account from machine B (already Paired)

* @CLOUD_LOGIN --> TN
* LOCAL ppm request all the existing segments from the cloud : account id + machine id + TN
* CLOUD Verify if the TN exists for the (account.id). Check if the machine id exists for the account id.
* CLOUD Prepare all the existing segments
* LOCAL ppm receive all the segments and merge them (the data inside the segment is never decoded)

#### EMAIL_CHANGE - The User changes its user name (email)

* CLOUD @CLOUD_LOGIN + The new email address is sent for verification process
* CLOUD On the cloud account the new email address is associated to the account number
* LOCAL The account.json is changed to match the new email

#### RECONNECT_AFTER_EMAIL_CHANGE - The User connects on machine B 

* CLOUD ppm starts and check all the account ID of account.json on the cloud to see if the email has changed!!
* LOCAL ppm replace the email in the account.json.
* From there it's a standard login

#### PASS_CHANGE - The User changes its OP (original password)

* LOCAL Encrypted files are rebuilt 
  * Open the original file with the former HASH(OP)
  * Put it in memory
  * The transaction are fully rebuilt
  * Recreate the file with the new UK
* CLOUD @CLOUD_LOGIN + The new DK is sent, processed and HDK is stored
* CLOUD Put the other machine.id in "REQUEST_CHANGE_PASSWORD" mode. The mode means we change the password on purpose.
* CLOUD Remove all the segments for the (account id) 
* LOCAL @CLOUD_SYNC : ppm and cloud exchange segments

#### The User deletes its last secret file, after a complete password change.

1. Login with the old password

@REQUEST_PASSWORD (UNSYNC_DK) :

* LOCAL The local login succeed
* CLOUD The cloud login fails (--> UNSYNC_DK)
* LOCAL The new password is requested to the user
* LOCAL Encrypted files are rebuilt
  * ...
* CLOUD @CLOUD_LOGIN + Change the account id status from UNSYNC_DK to VALID
* LOCAL @CLOUD_SYNC : ppm and cloud exchange segments

2. The user connects locally with the new password

@RESYNC_CRYPTO :
* The local login fails
* The cloud login succeed : if needed, change the account status from REQUEST_CHANGE_PASSWORD to VALID
* LOCAL Encrypted files are rebuilt
  * ...
* LOCAL @CLOUD_SYNC : ppm and cloud exchange segments

#### The User connects from a machine B, after the password was changed

1. The user connect locally with the old password
* CALL @REQUEST_PASSWORD (REQUEST_CHANGE_PASSWORD)

2. The user connects locally with the new password
* CALL @RESYNC_CRYPTO

#### Pairing  - We pair a machine B to the account, knowing the machine A is already active

// PPM is installed, so we have a machine.id
* Click the pairing icon
* Enter the email, account.id, and the OP
* Compute the UK and DK
* @CLOUD_LOGIN ( account, machine.id, DK ) -> TN + PENDING_PAIRING
*   CLOUD  Generate a temporary code, that the other machine can read
* LOCAL A : Grab the pairing code
* LOCAL B : TN + Send pairing code + pairing password
* CLOUD : Activate the account for the machine id
* CALL @CREATE_LOCAL_ACCOUNT => email, UK, secret_file, account.json
* @CLOUD_LOGIN()
* @CLOUD_SYNC(TN)

### Definitions 

* machine.id : used in the add_key end point to build the cloud uuid. It's not crypted nor secret.
* user account id : in the cloud account, it's associated with the email address, the ES and the (machine.id + SLK). 
        we get it everytime we login to the cloud, and send it with all the cloud request.

  
