#include <com/wrench.h>

#include "tchar.h"
#include "windows.h"  // IWYU pragma: keep

#pragma comment(lib, "Advapi32.lib")

std::string wstringToString(const std::wstring& wstr)
{
    // https://stackoverflow.com/questions/4804298/how-to-convert-wstring-into-string
    if (wstr.empty()) {
        return std::string();
    }

    int size = WideCharToMultiByte(CP_ACP, 0, &wstr[0], (int)wstr.size(), NULL,
                                   0, NULL, NULL);
    std::string ret = std::string(size, 0);
    WideCharToMultiByte(CP_ACP, 0, &wstr[0], (int)wstr.size(), &ret[0], size,
                        NULL, NULL);  // CP_UTF8

    return ret;
}

bool enumDetailsSerialPorts(std::vector<SerialPortInfo>& portInfoList)
{
    // https://msdn.microsoft.com/en-us/library/ms724256

#define MAX_KEY_LENGTH 255
#define MAX_VALUE_NAME 16383

    HKEY hKey;

    TCHAR achValue[MAX_VALUE_NAME];     // buffer for subkey name
    DWORD cchValue = MAX_VALUE_NAME;    // size of name string
    TCHAR achClass[MAX_PATH] = _T("");  // buffer for class name
    DWORD cchClassName = MAX_PATH;      // size of class string
    DWORD cSubKeys = 0;                 // number of subkeys
    DWORD cbMaxSubKey;                  // longest subkey size
    DWORD cchMaxClass;                  // longest class string
    DWORD cKeyNum;                      // number of values for key
    DWORD cchMaxValue;                  // longest value name
    DWORD cbMaxValueData;               // longest value data
    DWORD cbSecurityDescriptor;         // size of security descriptor
    FILETIME ftLastWriteTime;           // last write time

    int iRet = -1;
    bool bRet = false;

    std::string strPortName;
    SerialPortInfo m_serialPortInfo;

    TCHAR strDSName[MAX_VALUE_NAME];
    memset(strDSName, 0, MAX_VALUE_NAME);
    DWORD nBuffLen = 10;

    if (ERROR_SUCCESS == RegOpenKeyEx(HKEY_LOCAL_MACHINE,
                                      _T("HARDWARE\\DEVICEMAP\\SERIALCOMM"), 0,
                                      KEY_READ, &hKey)) {
        // Get the class name and the value count.
        iRet = RegQueryInfoKey(hKey,           // key handle
                               achClass,       // buffer for class name
                               &cchClassName,  // size of class string
                               NULL,           // reserved
                               &cSubKeys,      // number of subkeys
                               &cbMaxSubKey,   // longest subkey size
                               &cchMaxClass,   // longest class string
                               &cKeyNum,       // number of values for this key
                               &cchMaxValue,   // longest value name
                               &cbMaxValueData,        // longest value data
                               &cbSecurityDescriptor,  // security descriptor
                               &ftLastWriteTime);      // last write time

        if (!portInfoList.empty()) {
            portInfoList.clear();
        }

        // Enumerate the key values.
        if (cKeyNum > 0 && ERROR_SUCCESS == iRet) {
            for (int i = 0; i < (int)cKeyNum; i++) {
                cchValue = MAX_VALUE_NAME;
                achValue[0] = '\0';
                nBuffLen = MAX_KEY_LENGTH;

                if (ERROR_SUCCESS == RegEnumValue(hKey, i, achValue, &cchValue,
                                                  NULL, NULL, (LPBYTE)strDSName,
                                                  &nBuffLen)) {
#ifdef UNICODE
                    strPortName = wstringToString(strDSName);
#else
                    strPortName = std::string(strDSName);
#endif
                    m_serialPortInfo.portName = strPortName;
                    portInfoList.push_back(m_serialPortInfo);
                }
            }
        } else {
        }
    }

    if (portInfoList.empty()) {
        bRet = false;
    } else {
        bRet = true;
    }

    RegCloseKey(hKey);

    return bRet;
}

std::vector<SerialPortInfo> query_system_com_port()
{
    std::vector<SerialPortInfo> portInfoList;
    enumDetailsSerialPorts(portInfoList);
    return portInfoList;
}