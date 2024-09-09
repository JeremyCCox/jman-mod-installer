import GameDir from "./GameDir";
import {InstallerProfile} from "../lib/types.ts";
import {useState} from "react";
import LoadingSpinner from "./LoadingSpinner.tsx";

export default function UserLogin({handleSubmit,loading,config}:{handleSubmit:any,loading:boolean,config?:InstallerProfile}){
    const [username, setUsername] = useState(config?.sftpUsername||"")
    const [server, setServer] = useState(config?.sftpServer||"")
    const [port, setPort] = useState(config?.sftpPort||"")

    return(
        <div className={'text-center m-auto w-80'}>
            <form onSubmit={handleSubmit} className={'grid'}>
                <input type={'text'} name={'sftpUsername'} autoComplete={"username"} value={username} className={"py-3 my-2 "} placeholder={"username"} onChange={(e)=>{setUsername(e.currentTarget.value)}}/>
                <input type={'password'} name={'sftpPassword'} autoComplete={"current-password"} className={"py-3 my-2 "} placeholder={"password"} />
                <div className={"flex w-full"}>
                    <input type={'text'} name={"sftpServer"} className={"py-3 my-2 grow "} placeholder={"Profile Server"} value={server} onChange={(e)=>{setServer(e.currentTarget.value)}}/>
                    <input type={'text'} name={"sftpPort"} className={"py-3 my-2 w-24 "} placeholder={"Port"} value={port} onChange={(e)=>{setPort(e.currentTarget.value)}}/>
                </div>
                <GameDir/>
                <button disabled={loading}>
                    {loading?<LoadingSpinner/>:"Sign In"}
                </button>
            </form>
        </div>
    )
}