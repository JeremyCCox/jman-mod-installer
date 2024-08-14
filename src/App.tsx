import {QueryClient, QueryClientProvider, useQuery, useQueryClient, UseQueryResult} from "react-query";
import {FormEvent, useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api";
import {InstallerProfile} from "@my-types/*";
import UserLogin from "../components/UserLogin";
import InstallerBase from "../components/ModInstaller/InstallerBase.tsx";


function InstallerLauncher() {
    const queryClient = useQueryClient();
    const [loading,setLoading] = useState(false)
    const accessQuery=useQuery("login",async () => {
        return invoke("attempt_remote_connection_config").then((res)=>{
            return{success:res}
        }).catch(err=>{
            console.log(err)
        });
    })
    const configQuery:UseQueryResult<InstallerProfile> = useQuery(["config"],async () => {
        return await invoke("read_installer_config");
    },
        {enabled:!!accessQuery.data}
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
        setLoading(true)
        await queryClient.invalidateQueries("login")
        // console.log(config)
        await invoke("attempt_remote_connection_new",{installerConfig:config}).catch(err=>{
            console.error(err)
        })
        await queryClient.refetchQueries("login")
        return
    }

    // useEffect(()=>{
    //     console.log(configQuery.data)
    // },[configQuery])


    return (
        <>
            {/*<InstallerBase/>*/}
            <h1 className={'text-4xl font-bold font-mono mt-[15vh] mb-[5vh]'}>JMAN MOD MANAGER</h1>
            {accessQuery.isLoading?
                <div>
                    <h3 className={'absolute text-4xl top-[8vh] text-center w-full text-blue-300 animate-pulse'}>
                        Attempting login
                    </h3>
                </div>

                    :
                accessQuery.data?.success?
                    <InstallerBase/>
                    :
                    configQuery.data?
                        !accessQuery.data?.success?
                            <>
                                <h3 className={'absolute text-4xl top-[8vh] text-center w-full text-red-500 animate-pulse'}>
                                    Could not contact SFTP Server
                                </h3>
                                <UserLogin handleSubmit={handleSubmit} loading={loading} config={configQuery.data}/>
                            </>
                            :
                            <UserLogin handleSubmit={handleSubmit} loading={loading} config={configQuery.data}/>
                        :
                            <UserLogin handleSubmit={handleSubmit} loading={loading} />


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