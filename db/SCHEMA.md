# "Scheme" for the Underlying Database

## Scheme

- User:
  + email
  + pword\_hash
  + pword\_salt

- Folder
  + user\_email
  + name
  + parent

- Bookmark
  + user\_email
  + folder
  + name
  + url

- PasswordChangeRequest
  + user\_email
  + code

