import React from 'react'
import { Button, FormField } from '../../Shared'

type UserFormProps = {
  newUserName: string
  deleteUserData: boolean
  onNewUserNameChange: (value: string) => void
  onDeleteUserDataChange: (value: boolean) => void
  onCreate: () => void
}

function UserFormComponent({
  newUserName,
  deleteUserData,
  onNewUserNameChange,
  onDeleteUserDataChange,
  onCreate,
}: UserFormProps) {
  return (
    <div className="create-user">
      <FormField label="New user name">
        <input
          type="text"
          value={newUserName}
          onChange={(e) => onNewUserNameChange(e.target.value)}
          placeholder="Enter a display name"
        />
      </FormField>
      <Button variant="primary" onClick={onCreate}>
        Create &amp; switch
      </Button>
      <label className="toggle-row">
        <input
          type="checkbox"
          checked={deleteUserData}
          onChange={(e) => onDeleteUserDataChange(e.target.checked)}
        />
        <span>Delete data files when removing a user</span>
      </label>
    </div>
  )
}

export const UserForm = React.memo(UserFormComponent)
