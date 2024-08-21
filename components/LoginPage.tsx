import {useConfig} from "./contextHooks/configContext.tsx";
import {FormEvent, useState} from "react";
import {InstallerProfile} from "@my-types/*";
import LoadingSpinner from "./LoadingSpinner.tsx";
import UserLogin from "./UserLogin.tsx";
import {useNavigate} from "react-router-dom";

export default function LoginPage(){
    const config = useConfig();
    const [loading, setLoading] = useState(false)
    const navigate = useNavigate();
    const handleSubmit=async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault()
        console.log(e)
        setLoading(true)
        let profile:InstallerProfile = {defaultGameDir:e.currentTarget["defaultGameDir"].value,sftpUsername:e.currentTarget["sftpUsername"].value,sftpPassword:e.currentTarget["sftpPassword"].value,sftpServer:e.currentTarget["sftpServer"].value,sftpPort:e.currentTarget["sftpPort"].value}
        config.attemptLogin(profile).then(res=>{
            setLoading(false)
            if(res){
                navigate("/",{replace:true})
            }
        })
    }
    return(
        <>
            {config.configQuery.isLoading?
                <LoadingSpinner/>
                :
                    <UserLogin handleSubmit={handleSubmit} loading={loading} config={config.configQuery.data}/>
            }
        </>
    )
}