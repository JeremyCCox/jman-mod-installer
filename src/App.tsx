import {QueryClient, QueryClientProvider, useQuery, UseQueryResult} from "react-query";
import {FormEvent, useEffect} from "react";
import {invoke} from "@tauri-apps/api";
import {InstallerProfile} from "@my-types/*";
import UserLogin from "../components/UserLogin.tsx";
import InstallerBase from "../components/ModInstaller/InstallerBase.tsx";


function InstallerLauncher() {
    const accessQuery=useQuery(["login"],async () => {
        return invoke("attempt_remote_connection_config").then((res)=>{
            console.log("First Query")
            console.log(res)
            return{success:false}
        }).catch(err=>{
            console.log(err)
        });
    })
    const configQuery:UseQueryResult<InstallerProfile> = useQuery(["config"],async () => {
        console.log("Second Query")
        return await invoke("read_installer_config");
    },
        // {enabled:accessQuery.data?accessQuery.data.success:false}
    )
    useEffect(()=>{
        console.log(accessQuery)
    },[accessQuery])

    const handleSubmit=async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault()
        let config:InstallerProfile = {defaultGameDir:e.currentTarget["defaultGameDir"].value,sftpUsername:e.currentTarget["sftpUsername"].value,sftpPassword:e.currentTarget["sftpPassword"].value,sftpServer:e.currentTarget["sftpServer"].value,sftpPort:e.currentTarget["sftpPort"].value}
        await attemptLogin(config)
    }

    const attemptLogin=async (config:InstallerProfile)=>{
        // console.log(config)
        console.log(await invoke("attempt_remote_connection_new",{installerConfig:config}))
        return
    }

    // useEffect(()=>{
    //     console.log(configQuery.data)
    // },[configQuery])


    return (
        <>

            {accessQuery.isLoading?
                <div>
                    <h3 className={'absolute text-4xl top-[8vh] text-center w-full text-blue-300 animate-pulse'}>
                        Attempting login
                    </h3>
                </div>
                    :
                accessQuery.data?
                    <InstallerBase/>
                    :
                    configQuery.data?
                            <UserLogin handleSubmit={handleSubmit} config={configQuery.data}/>
                        :
                            <UserLogin handleSubmit={handleSubmit} />


            }
        </>
        );
}

export default function App(){
    const queryClient = new QueryClient({defaultOptions:{
        queries:{
            refetchOnWindowFocus:false,
        }
        }})
    return(
        <>
            <QueryClientProvider client={queryClient}>
                <InstallerLauncher />
            </QueryClientProvider>
            {/*<InstallerBase/>*/}
        </>
    )
}