import {createContext, ReactNode, useContext} from "react";
import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";
import {InstallerProfile} from "@my-types/*";
import {Navigate} from "react-router-dom";

const ConfigContext = createContext({});

export function ConfigProvider({children}:Readonly<{ children?:ReactNode }>){
    const queryClient = useQueryClient();
    const accessQuery=useQuery("login",async () => {
        return invoke<boolean>("attempt_remote_connection_config").then((res)=>{
            return{success:res}
        }).catch(err=>{
            console.log(err)
            return({success:false})
        });
    })

    const configQuery:UseQueryResult<InstallerProfile> = useQuery(["config"],async () => {
            return await invoke("read_installer_config");
        },
        // {enabled:!!accessQuery.data}
    )
    const logout = async ()=>{
        await queryClient.invalidateQueries("login")
        let config ={} as InstallerProfile;
        await invoke("write_installer_config",{installerConfig:config}).catch(err=>{
            console.error(err)
        })
        await queryClient.refetchQueries("login")
    }
    const updateConfig = async (config:InstallerProfile)=>{
        await queryClient.invalidateQueries("login")
        await invoke("write_installer_config",{installerConfig:config}).catch(err=>{
            console.error(err)
        })
        await queryClient.refetchQueries("login")
    }
    const attemptLogin=async (config:InstallerProfile)=>{
        await queryClient.invalidateQueries("login")
        // console.log(config)
        let result = await invoke("attempt_remote_connection_new",{installerConfig:config}).catch(err=>{
            console.error(err)
            return false
        })
        await queryClient.refetchQueries("login")
        console.log(result)
        return result
    }

    return(
        <ConfigContext.Provider value={{
            accessQuery,
            configQuery,
            attemptLogin,
            logout,
        }}>
            <>
                {children}
            </>
        </ConfigContext.Provider>
    )
}
export function useConfig(){
    return useContext(ConfigContext)
}
export function ConfigValid({children}:Readonly<{children?:ReactNode}>){
    const config = useConfig();
    console.log(config.accessQuery)
    return(
        <>
            {config.accessQuery.isLoading?
                <div>
                    <h3 className={'absolute text-4xl top-[8vh] text-center w-full text-blue-300 animate-pulse'}>
                        Attempting login
                    </h3>
                </div>
                :
                config.accessQuery.data?.success?
                    <>
                        {children}
                    </>
                    :
                    <Navigate to={"/login"} replace={true}/>
            }
            </>
    )


}